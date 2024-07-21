extern crate libc;

use expect_test::expect;
use pie_lua::luajit_module;
use std::ffi::CStr;

#[luajit_module]
mod exports {
    #[no_mangle]
    pub extern "C" fn my_c_function(a: i32, b: f64) -> i32 {
        a + b as i32
    }

    #[no_mangle]
    pub extern "C" fn another_c_function(x: i32) -> i32 {
        x * 2
    }
}

#[test]
fn test_luajit_macro() {
    // Call get_lua_ffi_decls to get the FFI declarations
    let decls_ptr = crate::exports::exports_luajit_ffi_decls();
    let decls = unsafe { CStr::from_ptr(decls_ptr).to_str().unwrap() };

    // Use expect-test to check the output
    let expected = expect![[r#"
        int32_t my_c_function(int32_t a, double b);
        int32_t another_c_function(int32_t x);"#]];
    expected.assert_eq(decls);

    let lua = unsafe { mlua::Lua::unsafe_new() };
    let x: (String, i32) = lua
        .load(
            r#"
        local ffi = require "ffi"

        ffi.cdef [[
            const char* exports_luajit_ffi_decls();
        ]]

        local decls = ffi.string(ffi.C.exports_luajit_ffi_decls())
        ffi.cdef(decls)
        print(decls)
        print(ffi.C.another_c_function(123))
        return decls, ffi.C.another_c_function(123)
    "#,
        )
        .eval()
        .unwrap();
    expect![[r#"
        (
            "int32_t my_c_function(int32_t a, double b);\nint32_t another_c_function(int32_t x);",
            246,
        )"#]]
    .assert_eq(&format!("{x:#?}"));
}
