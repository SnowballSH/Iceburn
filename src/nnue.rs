use lazy_static::lazy_static;
use libloading;

lazy_static! {
    static ref NNUE: libloading::Library = unsafe {
        libloading::Library::new("./libnnueprobe.dll").unwrap_or_else(|x| panic!("{}", x))
    };
    static ref NNUE_INIT: libloading::Symbol<'static, unsafe extern "C" fn(*const u8)> =
        unsafe { NNUE.get(b"nnue_init").unwrap() };
    static ref NNUE_EVAL_FEN: libloading::Symbol<'static, unsafe extern "C" fn(*const u8) -> i32> =
        unsafe { NNUE.get(b"nnue_evaluate_fen").unwrap() };
}

pub fn nnue_init(eval_file: &str) {
    unsafe {
        NNUE_INIT(eval_file.as_ptr());
    }
}

pub fn nnue_eval_fen(fen: &str) -> i32 {
    unsafe { NNUE_EVAL_FEN(fen.as_ptr()) }
}
