local module = {}

--- A future represents a value might not be available now, but might be available later.
--- When the value becomes available, the future is *resolved*.
--- You can get the value of a future by using 'await'
local FutureImpl = { future = true }
export type Future<Res> = { r: Res } & typeof(FutureImpl)

--- Wait for the future to be resolved, and get its value
--- If this is called inside Update(), it might block rendering.
--- Be careful, some futures might never be resolved!
--- Returns instantly if the future is already resolved
function FutureImpl.await<Res>(p: Future<Res>): Res
	error("Not implemented")
end

--- If the future is resolved, return its value, otherwise return the default value
function FutureImpl.getOr<Res>(p: Future<Res>, default: Res): Res
    error("Not implemented")
end

--- Create a future that is already resolved with the given value
function module.resolved<Res>(res: Res): Future<Res>
	error("Not implemented")
end

--- Create a future that will be resolved when the callback calls resolve
--- You can optionally provide a tag value to help with type inference, the tag is unused at runtime.
function module.future<Res>(fn: (resolve: () -> Res) -> (), tag: Res?): Future<Res>
    error("Not implemented")
end

function FutureImpl.flatten<Res>(p: Future<Future<Res>>): Future<Res>
    error("Not implemented")
end

--- Returns true if the future is resolved and its value is available
function FutureImpl.isReady<Res>(p: Future<Res>): boolean
	error("Not implemented")
end

function FutureImpl.flatMap<A, B>(p: Future<A>, fn: (A) -> Future<B>): Future<B>
	error("Not implemented")
end

function FutureImpl.andThen<Res>(p: Future<Res>, fn: (Res) -> ()): Future<Res>
	error("Not implemented")
end

return module
