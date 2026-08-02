use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Wake, Waker};

use futures::future::Either;

use crate::console::{print_err, print_warn};
use crate::game_resource::{self, Resource, ResourceManager, Status};
use crate::lua_env::{LuaHandle, print_lua_error_from_error};
use ouroboros::self_referencing;
use vectarine_plugin_sdk::mlua;
use vectarine_plugin_sdk::mlua::function::AsyncCallFuture;

pub struct DummyWaker;

impl Wake for DummyWaker {
    fn wake(self: Arc<Self>) {
        // In a fully featured custom runtime, this would push your future
        // to a "ready" queue to be polled again.
        // For a simple frame-by-frame game loop, a no-op is perfectly fine.
    }
}

/// Turn a value of type T into a future that is immediately ready with that value.
pub fn futurify<T>(t: T) -> impl Future<Output = T> {
    std::future::ready(t)
}

struct ResourceFuture<T: Resource + 'static> {
    manager: Rc<ResourceManager>,
    resource_path: PathBuf,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Resource + 'static> Future for ResourceFuture<T> {
    type Output = Option<Rc<T>>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let id = self
            .manager
            .get_id_by_path(&self.resource_path)
            .expect("Tkt");
        let resource_holder = self.manager.get_holder_by_id(id);
        match resource_holder.get_status() {
            Status::Loading => std::task::Poll::Pending,
            Status::Unloaded => std::task::Poll::Pending,
            Status::Error(_) => std::task::Poll::Ready(None),
            Status::Loaded => {
                let res = resource_holder.get_underlying_resource::<T>();
                if let Ok(res) = res {
                    std::task::Poll::Ready(Some(res))
                } else {
                    std::task::Poll::Ready(None)
                }
            }
        }
    }
}

/// Create a future that is resolved when the resource provided is loaded / in an error state.
pub fn make_resource_future<T: Resource + 'static>(
    manager: Rc<ResourceManager>,
    loading_cause_path: Option<&Path>,
    target_table: vectarine_plugin_sdk::mlua::Table,
    resource_path: PathBuf,
) -> impl Future<Output = Option<Rc<T>>> {
    let resource_path =
        game_resource::resolve_dot_relative_paths(&resource_path, loading_cause_path);
    let id_opt = manager.get_id_by_path(&resource_path);
    if let Some(id) = id_opt {
        let holder = manager.get_holder_by_id(id);
        if holder.get_status() == Status::Loaded {
            if let Ok(res) = holder.get_underlying_resource::<T>() {
                return Either::Left(std::future::ready(Some(res)));
            } else {
                return Either::Left(std::future::ready(None));
            }
        } else if let Status::Error(_) = holder.get_status() {
            return Either::Left(std::future::ready(None));
        }
    } else {
        manager.schedule_load_script_resource(
            resource_path.as_path(),
            loading_cause_path,
            target_table,
        );
    }
    Either::Right(ResourceFuture {
        manager,
        resource_path,
        _marker: std::marker::PhantomData,
    })
}

#[self_referencing]
struct AsyncHandlerInternal {
    future_queue: Vec<Pin<Box<dyn Future<Output = Result<(), mlua::Error>>>>>,
    waker: Waker,
    #[borrows(waker)]
    #[not_covariant]
    ctx: Context<'this>,
}

/// A simple single-threaded async runtime for Lua coroutines.
pub struct AsyncLuaHandle {
    internal: AsyncHandlerInternal,
}

impl Default for AsyncLuaHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncLuaHandle {
    pub fn new() -> Self {
        let waker = Waker::from(Arc::new(DummyWaker));
        let internal = AsyncHandlerInternalBuilder {
            future_queue: Vec::new(),
            waker,
            ctx_builder: |w| Context::from_waker(w),
        }
        .build();

        AsyncLuaHandle { internal }
    }

    /// Schedule a future to be executed at a later time.
    /// Errors produced by the future will be printed to the console.
    /// If the future is ready, returns true.
    pub fn schedule_future(
        &mut self,
        lua_handle: &Rc<LuaHandle>,
        mut future: Pin<Box<dyn Future<Output = Result<(), mlua::Error>>>>,
    ) -> bool {
        let result = self.internal.with_ctx_mut(|ctx| future.as_mut().poll(ctx));

        match result {
            std::task::Poll::Ready(Ok(_)) => true,
            std::task::Poll::Ready(Err(err)) => {
                print_err("Something when wrong during the execution of a future.".to_string());
                print_lua_error_from_error(lua_handle, &err);
                true
            }
            std::task::Poll::Pending => {
                self.internal.with_future_queue_mut(|queue| {
                    queue.push(future);
                });
                false
            }
        }
    }

    pub fn are_futures_pending(&self) -> bool {
        self.internal.with_future_queue(|queue| !queue.is_empty())
    }

    /// Schedule the lua coroutine to be executed at a later time.
    /// Errors produced by the coroutine will be printed to the console.
    pub fn execute_frame(
        &mut self,
        lua_handle: &Rc<LuaHandle>,
        future: AsyncCallFuture<()>,
        // If true, a warning will be printed if the future is not ready yet.
        // Typically, during calls to update, async calls should be ready, and if they are not, rendering got interrupted.
        print_warning_if_pending: bool,
    ) {
        let mut pinned = Box::pin(future);
        let polled = self.internal.with_ctx_mut(|ctx| pinned.as_mut().poll(ctx));
        match polled {
            std::task::Poll::Ready(Ok(_)) => {}
            std::task::Poll::Ready(Err(err)) => {
                print_err("Something when wrong during the execution of a future.".to_string());
                print_lua_error_from_error(lua_handle, &err);
            }
            std::task::Poll::Pending => {
                if print_warning_if_pending {
                    print_warn("An asynchronous function was called during a critical section. This can cause half-drawn frames to render.".to_string());
                }
                self.schedule_future(lua_handle, pinned);
            }
        }
    }

    /// This function needs to be called regularly to update running futures, usually once per frame.
    /// Do not call it directly, call `LuaHandle::poll_pending_futures` instead.
    pub fn poll_pending_futures(&mut self, lua_handle: &Rc<LuaHandle>) {
        self.internal.with_mut(|fields| {
            fields.future_queue.retain_mut(|future| {
                let polled = future.as_mut().poll(fields.ctx);
                match polled {
                    std::task::Poll::Ready(Ok(_)) => false,
                    std::task::Poll::Ready(Err(err)) => {
                        print_err(
                            "Something when wrong during the execution of a future.".to_string(),
                        );
                        print_lua_error_from_error(lua_handle, &err);
                        false
                    }
                    std::task::Poll::Pending => true,
                }
            });
        });
    }
}
