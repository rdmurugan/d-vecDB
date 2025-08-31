use sqlite_loadable::prelude::*;
use sqlite_loadable::table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor};
use sqlite_loadable::Error;

use crate::cosine::cosine_blob_f32;
use crate::{get_qvec, set_vtab_config, get_vtab_config};

// Raw SQLite shims from the crate (alpha.6)
use sqlite_loadable::ext::{
    sqlite3, sqlite3_stmt, sqlite3_value,
    sqlite3ext_prepare_v2, sqlite3ext_step, sqlite3ext_finalize,
    sqlite3ext_column_int64, sqlite3ext_column_value,
};
use sqlite_loadable::{SQLITE_ROW, SQLITE_DONE};

use std::ffi::CString;

/// Per-table state (one per CREATE VIRTUAL TABLE)
#[derive(Debug)]
pub struct PocketVecTable {
    db: *mut sqlite3,
    base_table: Box<str>,   // e.g., "docs_vecs"
    dim: usize,           // embedding dimension (float32 => blob len = 4*dim)
}

/// Cursor over results
#[derive(Debug)]
pub struct PocketVecCursor {
    db: *mut sqlite3,
    base_table: String,
    dim: usize,
    rows: Vec<(i64, f64)>, // (id, distance)
    pos: usize,
}

/// Infer dimension from the first row in base_table by inspecting length(vec)
fn infer_dim(db: *mut sqlite3, base_table: &str) -> Option<usize> {
    let sql = format!("SELECT length(vec) FROM {} LIMIT 1", base_table);
    let csql = CString::new(sql).ok()?;
    let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();

    unsafe {
        if sqlite3ext_prepare_v2(db, csql.as_ptr(), -1, &mut stmt, std::ptr::null_mut()) != 0 {
            return None;
        }
        let mut out: Option<usize> = None;
        loop {
            let rc = sqlite3ext_step(stmt);
            if rc == SQLITE_ROW {
                let bytes = sqlite3ext_column_int64(stmt, 0);
                if bytes > 0 && (bytes % 4) == 0 {
                    out = Some((bytes as usize) / 4);
                }
                break;
            } else if rc == SQLITE_DONE {
                break;
            } else {
                sqlite3ext_finalize(stmt);
                return None;
            }
        }
        sqlite3ext_finalize(stmt);
        out
    }
}

/// Parse args; supports `key=value` and `key value`, commas/parentheses tolerated.
fn parse_args(args: &VTabArguments) -> (String, usize) {
    let mut base = "docs_vecs".to_string();
    let mut dim: usize = 0;

    // Skip fewer elements - the arguments structure seems different than expected
    let skip_count = if args.arguments.len() <= 3 { 0 } else { 3 };
    let tail = args.arguments.iter().skip(skip_count).cloned().collect::<Vec<_>>().join(" ");
    log::debug!("Parsing args - raw tail: '{}', all args: {:?}, skip_count: {}", tail, args.arguments, skip_count);
    let normalized = tail.replace(['(', ')', ','], " ");
    log::debug!("Parsing args - normalized: '{}'", normalized);
    let toks: Vec<&str> = normalized.split_whitespace().collect();
    log::debug!("Parsing args - tokens: {:?}", toks);

    let mut i = 0;
    while i < toks.len() {
        let t = toks[i];

        if let Some(eq) = t.find('=') {
            let (k, v) = t.split_at(eq);
            let v = &v[1..];
            match k.trim() {
                "base_table" => if !v.is_empty() { base = v.trim().to_string(); },
                "dim" => dim = v.trim().parse::<usize>().unwrap_or(0),
                _ => {}
            }
            i += 1;
            continue;
        }

        if t == "base_table" && i + 1 < toks.len() {
            base = toks[i + 1].trim().to_string();
            i += 2;
            continue;
        }
        if t == "dim" && i + 1 < toks.len() {
            dim = toks[i + 1].trim().parse::<usize>().unwrap_or(0);
            i += 2;
            continue;
        }

        i += 1;
    }
    
    log::debug!("Parsing args - final base_table: '{}', dim: {}", base, dim);
    (base, dim)
}

impl<'vtab> VTab<'vtab> for PocketVecTable {
    type Aux = ();
    type Cursor = PocketVecCursor;

    fn connect(
        db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> sqlite_loadable::Result<(String, Self)> {
        log::debug!("connect() called with args (len={}): {:?}", args.arguments.len(), args.arguments);
        let (base, mut dim) = parse_args(&args);
        log::debug!("connect() parsed base_table: '{}', dim: {}", base, dim);

        // If dim wasn't provided, try to infer it from the base table
        if dim == 0 {
            if let Some(d) = infer_dim(db, &base) {
                dim = d;
            }
        }

        if dim == 0 {
            return Err(Error::new_message(
                "pocketvec: dim must be > 0 (pass dim=..., or ensure base_table has at least one vec row to infer)",
            ));
        }

        // IMPORTANT: don't declare `rowid` as a column. Let SQLite ask for rowid via rowid().
        // Schema columns (index 0,1): id, distance
        let schema = "CREATE TABLE x(id INTEGER, distance REAL)".to_string();

        // Store config in thread-local storage as workaround for sqlite-loadable bug
        set_vtab_config(base.clone(), dim, db);
        
        let table = PocketVecTable { 
            db, 
            base_table: base.into_boxed_str(), 
            dim 
        };
        log::debug!("connect() returning table with base_table: '{}', dim: {}", table.base_table, table.dim);
        Ok((schema, table))
    }

    fn best_index(&self, mut info: IndexInfo) -> Result<(), BestIndexError> {
        // Linear scan cost hint
        info.set_estimated_cost(1e6_f64);
        Ok(())
    }

    fn open(&'vtab mut self) -> sqlite_loadable::Result<Self::Cursor> {
        log::debug!("open() called, self={:?}", self);
        
        // Get config from thread-local storage as workaround for sqlite-loadable bug
        let (base_table, dim, db) = get_vtab_config()
            .unwrap_or(("docs_vecs".to_string(), 128, std::ptr::null_mut()));  // fallback defaults
        
        let cursor = PocketVecCursor {
            db,
            base_table,
            dim,
            rows: Vec::new(),
            pos: 0,
        };
        log::debug!("open() created cursor={:?}", cursor);
        Ok(cursor)
    }
}

impl VTabCursor for PocketVecCursor {
    fn filter(
        &mut self,
        _idx_num: std::os::raw::c_int,
        _idx_str: Option<&str>,
        _args: &[*mut sqlite3_value],
    ) -> sqlite_loadable::Result<()> {
        log::debug!("filter() called with base_table: '{}', dim: {}", self.base_table, self.dim);
        
        // Query vector from thread-local state
        let q = get_qvec();
        if q.is_empty() {
            return Err(Error::new_message("pocketvec: set query via pocketvec_set_q(?) first"));
        }
        if q.len() != 4 * self.dim {
            return Err(Error::new_message("pocketvec: query vector length != 4*dim"));
        }
        
        log::debug!("Query vector validated, dim={}, qlen={}", self.dim, q.len());

        self.rows.clear();

        // Scan the base table
        let sql = format!("SELECT rowid, vec FROM {}", self.base_table);
        log::debug!("SQL query: {}", sql);
        
        let csql = match CString::new(sql) {
            Ok(s) => s,
            Err(_) => return Err(Error::new_message("pocketvec: invalid SQL string")),
        };
        let mut stmt: *mut sqlite3_stmt = std::ptr::null_mut();
        log::debug!("About to prepare statement");

        unsafe {
            if sqlite3ext_prepare_v2(self.db, csql.as_ptr(), -1, &mut stmt, std::ptr::null_mut()) != 0 {
                return Err(Error::new_message("pocketvec: prepare failed"));
            }
            log::debug!("Statement prepared successfully");

            // Accumulate an error string, but finalize exactly once at the end.
            let mut err: Option<&'static str> = None;

            loop {
                let rc = sqlite3ext_step(stmt);
                log::debug!("Step result: {}", rc);
                if rc == SQLITE_ROW {
                    let id = sqlite3ext_column_int64(stmt, 0);
                    log::debug!("Processing row id: {}", id);

                    // Get BLOB data via sqlite3_value and immediately copy to owned Vec
                    let val = sqlite3ext_column_value(stmt, 1);
                    log::debug!("Got column value pointer: {:p}", val);
                    let vb_slice = sqlite_loadable::api::value_blob(&val);
                    log::debug!("Got blob slice, length: {}", vb_slice.len());
                    // CRITICAL: copy the data immediately while the value is still valid
                    let vb: Vec<u8> = vb_slice.to_vec();
                    log::debug!("Copied to Vec, length: {}", vb.len());

                    if vb.len() != 4 * self.dim {
                        err = Some("pocketvec: row vector length != 4*dim");
                        break;
                    }

                    // Score and push
                    match cosine_blob_f32(&q, &vb) {
                        Ok(sim) => {
                            let dist = (1.0 - sim) as f64;
                            self.rows.push((id, dist));
                        }
                        Err(_) => {
                            err = Some("pocketvec: cosine error");
                            break;
                        }
                    }
                } else if rc == SQLITE_DONE {
                    break;
                } else {
                    err = Some("pocketvec: step failed");
                    break;
                }
            }

            sqlite3ext_finalize(stmt);

            if let Some(m) = err {
                return Err(Error::new_message(m));
            }
        }

        // Sort by distance asc
        self.rows
            .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        self.pos = 0;
        Ok(())
    }

    fn next(&mut self) -> sqlite_loadable::Result<()> {
        self.pos += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.pos >= self.rows.len()
    }

    fn column(
        &self,
        ctx: *mut sqlite3_context,
        i: std::os::raw::c_int,
    ) -> sqlite_loadable::Result<()> {
        // Columns: 0 = id, 1 = distance (schema: "id INTEGER, distance REAL")
        if self.pos >= self.rows.len() {
            // Defensive: shouldn't be called when eof() == true.
            sqlite_loadable::api::result_null(ctx);
            return Ok(());
        }
        match i {
            0 => sqlite_loadable::api::result_int64(ctx, self.rows[self.pos].0),
            1 => sqlite_loadable::api::result_double(ctx, self.rows[self.pos].1),
            _ => sqlite_loadable::api::result_null(ctx),
        }
        Ok(())
    }

    fn rowid(&self) -> sqlite_loadable::Result<i64> {
        // Report SQLite rowid as our id
        Ok(self.rows[self.pos].0)
    }
}
