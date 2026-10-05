use mlua::Lua;
use crate::error::{KonachaError, Result};

/// Runs a user-supplied Lua hook. Sandboxed: `io`, `os`, `package`, `debug`,
/// `dofile`, `loadfile`, `require`, `load` are removed from globals before
/// the script is loaded.
pub fn run_format_hook(script: &str, note_json: &str) -> Result<String> {
    let lua = Lua::new();
    {
        let g = lua.globals();
        for banned in ["io", "os", "package", "debug", "dofile", "loadfile", "require", "load"] {
            g.set(banned, mlua::Value::Nil).map_err(|e| KonachaError::Script(e.to_string()))?;
        }
    }

    lua.load(script).exec().map_err(|e| KonachaError::Script(e.to_string()))?;

    let f: mlua::Function = lua.globals().get("format")
    .map_err(|_| KonachaError::Script("script must define `function format(note)`".into()))?;

    let out: String = f.call(note_json.to_string())
    .map_err(|e| KonachaError::Script(e.to_string()))?;
    Ok(out)
}

pub const SAMPLE_HOOK: &str = r#"-- Konacha Lua hook: runs after every save.
-- `note` is a JSON string; must return a JSON string.
function format(note)
return note
end
"#;
