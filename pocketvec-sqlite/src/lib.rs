use sqlite_loadable::prelude::*;
use sqlite_loadable::{define_scalar_function, Error};
use sqlite_loadable::table;

mod cosine;
mod vtab;

// -------------------- query vector state (thread-local) --------------------
use std::cell::RefCell;

thread_local! {
    static QVEC: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static VTAB_CONFIG: RefCell<Option<(String, usize, *mut sqlite3)>> = RefCell::new(None);
}

pub fn set_qvec(bytes: &[u8]) {
    QVEC.with(|cell| {
        let mut v = cell.borrow_mut();
        v.clear();
        v.extend_from_slice(bytes);
    });
}
pub fn get_qvec() -> Vec<u8> {
    QVEC.with(|cell| cell.borrow().clone())
}

pub fn set_vtab_config(base_table: String, dim: usize, db: *mut sqlite3) {
    VTAB_CONFIG.with(|cell| {
        *cell.borrow_mut() = Some((base_table, dim, db));
    });
}

pub fn get_vtab_config() -> Option<(String, usize, *mut sqlite3)> {
    VTAB_CONFIG.with(|cell| cell.borrow().clone())
}

// -------------------- scalar functions --------------------

fn pocketvec_version(ctx: *mut sqlite3_context, _args: &[*mut sqlite3_value]) -> sqlite_loadable::Result<()> {
    let _ = sqlite_loadable::api::result_text(ctx, "pocketvec-sqlite 0.1.0");
    Ok(())
}

fn pocketvec_cosine(ctx: *mut sqlite3_context, args: &[*mut sqlite3_value]) -> sqlite_loadable::Result<()> {
    let a = sqlite_loadable::api::value_blob(&args[0]);
    let b = sqlite_loadable::api::value_blob(&args[1]);
    let sim = match cosine::cosine_blob_f32(a, b) {
        Ok(v) => v,
        Err(_) => return Err(Error::new_message("cosine error")),
    };
    let _ = sqlite_loadable::api::result_double(ctx, sim as f64);
    Ok(())
}

// Set the thread-local query vector used by the vtab
fn pocketvec_set_q(ctx: *mut sqlite3_context, args: &[*mut sqlite3_value]) -> sqlite_loadable::Result<()> {
    if args.is_empty() { return Err(Error::new_message("pocketvec_set_q: missing argument")); }
    let q = sqlite_loadable::api::value_blob(&args[0]);
    if q.len() % 4 != 0 {
        return Err(Error::new_message("pocketvec_set_q: blob must be float32[]"));
    }
    set_qvec(q);
    let _ = sqlite_loadable::api::result_int(ctx, 1);
    Ok(())
}

// -------------------- entrypoint --------------------

#[sqlite_entrypoint]
pub fn sqlite3_extension_init(db: *mut sqlite3) -> sqlite_loadable::Result<()> {
    let _ = env_logger::try_init();

    define_scalar_function(db, "pocketvec_version", 0, pocketvec_version, FunctionFlags::DETERMINISTIC)?;
    define_scalar_function(db, "pocketvec_cosine", 2, pocketvec_cosine, FunctionFlags::DETERMINISTIC)?;
    define_scalar_function(db, "pocketvec_set_q", 1, pocketvec_set_q, FunctionFlags::DETERMINISTIC)?;

    // Register the vtab: name = "pocketvec"
    table::define_virtual_table::<vtab::PocketVecTable>(db, "pocketvec", None)?;
    Ok(())
}
