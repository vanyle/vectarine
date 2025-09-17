#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl mlua::FromLua for Vec2 {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => unreachable!(),
        }
    }
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn with_x(self, x: f32) -> Self {
        Self { x, y: self.y }
    }
    pub fn with_y(self, y: f32) -> Self {
        Self { x: self.x, y }
    }
}

impl mlua::UserData for Vec2 {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, v| Ok(v.x));
        fields.add_field_method_get("y", |_, v| Ok(v.y));
        fields.add_field_method_set("x", |_, vec, v| {
            vec.x = v;
            Ok(())
        });
        fields.add_field_method_set("y", |_, vec, v| {
            vec.y = v;
            Ok(())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("length", |_, vec, ()| {
            Ok((vec.x * vec.x + vec.y * vec.y).sqrt())
        });
        methods.add_method("scale", |_, vec, (k,): (f32,)| {
            Ok(Vec2 {
                x: vec.x * k,
                y: vec.y * k,
            })
        });
        methods.add_method("cmul", |_, vec, (other,): (Vec2,)| {
            Ok(Vec2 {
                x: vec.x * other.x - vec.y * other.y,
                y: vec.y * other.x + vec.x * other.y,
            })
        });
        methods.add_meta_function(mlua::MetaMethod::Add, |_, (vec, other): (Vec2, Vec2)| {
            Ok(Vec2 {
                x: vec.x + other.x,
                y: vec.y + other.y,
            })
        });
        methods.add_meta_function(mlua::MetaMethod::Sub, |_, (vec, other): (Vec2, Vec2)| {
            Ok(Vec2 {
                x: vec.x - other.x,
                y: vec.y - other.y,
            })
        });
        methods.add_meta_function(mlua::MetaMethod::Mul, |_, (vec, other): (Vec2, Vec2)| {
            Ok(Vec2 {
                x: vec.x * other.x,
                y: vec.y * other.y,
            })
        });
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, vec, _any: mlua::Value| {
            Ok(format!("V2({}, {})", vec.x, vec.y))
        });
    }
}

pub fn setup_vec2_api(lua: &mlua::Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    globals.set(
        "V2",
        lua.create_function(|lua, (x, y): (f32, f32)| {
            let data = mlua::Value::UserData(lua.create_userdata(Vec2 { x, y })?);
            Ok(data)
        })?,
    )?;
    Ok(())
}
