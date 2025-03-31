use std::time::Instant;
use mlua::{AnyUserData,Lua, UserDataMethods};

#[derive(Clone)]
pub struct TimerContext {
    pub timer: Instant,  
}

impl mlua::UserData for TimerContext {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("elapsed", |_, this, _: ()| {
            let elapsed = this.timer.elapsed();
            Ok(elapsed.as_secs_f64())
        });

        methods.add_method_mut("reset", |_, this, _: ()| {
            this.timer = Instant::now();
            Ok(())
        });

    }
}

pub fn create_instant_timer(lua: &Lua, _: ()) -> Result<AnyUserData, mlua::Error> {
    let timer = TimerContext {
        timer: Instant::now(),
    };
    lua.create_userdata(timer)
}

pub fn init_timer_methods(lua: &Lua) -> Result<(), mlua::Error> {
    let globals = lua.globals();
    globals.set("create_timer", lua.create_function(create_instant_timer)?)?;
    Ok(())
}