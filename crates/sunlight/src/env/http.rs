use crate::http::{self, Request, Response};
use crate::lua::*;
use crate::task;

pub unsafe extern "C-unwind" fn request(l: State) -> i32 {
    unsafe {
        let mut headers = Vec::new();
        lua_pushnil(l);
        while lua_next(l, 3) != 0 {
            headers.push((string(l, -2), string(l, -1)));
            lua_settop(l, -2);
        }
        let request = Request {
            url: string(l, 1),
            method: string(l, 2),
            headers,
            body: match lua_type(l, 4) {
                LUA_TNIL => Vec::new(),
                _ => bytes(l, 4).to_vec(),
            },
        };
        task::suspend(l, move || {
            let response = http::send(&request);
            Box::new(move |l| push(l, response))
        })
    }
}

/// `HttpGet(url)`, also reached as `game:HttpGet(url)`.
pub unsafe extern "C-unwind" fn httpget(l: State) -> i32 {
    unsafe {
        let index = if lua_type(l, 1) == LUA_TSTRING { 1 } else { 2 };
        let url = string(l, index);
        if !url.starts_with("http://") && !url.starts_with("https://") {
            error(l, c"HttpGet: expected an http:// or https:// URL");
        }
        let request = Request {
            url,
            method: "GET".into(),
            headers: Vec::new(),
            body: Vec::new(),
        };
        task::suspend(l, move || {
            let body = match http::send(&request) {
                Ok(response) => response.body,
                Err(error) => error.to_string().into_bytes(),
            };
            Box::new(move |l| {
                push_bytes(l, &body);
                1
            })
        })
    }
}

unsafe fn push(l: State, response: Result<Response, curl::Error>) -> i32 {
    let response = response.unwrap_or_else(|error| Response {
        message: error.to_string(),
        ..Default::default()
    });
    unsafe {
        lua_createtable(l, 0, 5);
        lua_pushboolean(l, (200..300).contains(&response.status) as i32);
        lua_setfield(l, -2, c"Success".as_ptr());
        lua_pushinteger(l, response.status as i32);
        lua_setfield(l, -2, c"StatusCode".as_ptr());
        push_bytes(l, response.message.as_bytes());
        lua_setfield(l, -2, c"StatusMessage".as_ptr());
        push_bytes(l, &response.body);
        lua_setfield(l, -2, c"Body".as_ptr());
        lua_createtable(l, 0, response.headers.len() as i32);
        for (name, value) in &response.headers {
            push_bytes(l, name.as_bytes());
            push_bytes(l, value.as_bytes());
            lua_rawset(l, -3);
        }
        lua_setfield(l, -2, c"Headers".as_ptr());
    }
    1
}

unsafe fn string(l: State, index: i32) -> String {
    String::from_utf8_lossy(unsafe { bytes(l, index) }).into_owned()
}
