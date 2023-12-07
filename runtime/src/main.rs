use std::{
    env,
    fs::File,
    io::{self, BufReader, Read, Seek},
};

use rlua::{InitFlags, Result, StdLib};

const INDEX_SIZE: usize = 8;

fn open_file() -> io::Result<File> {
    let this = env::current_exe()?;
    File::open(this)
}

fn get_size() -> io::Result<i64> {
    let file = open_file()?;
    let mut bf = BufReader::new(file);
    bf.seek(io::SeekFrom::End(-(INDEX_SIZE as i64)))?;
    let mut size = [0; INDEX_SIZE];
    bf.read_exact(&mut size)?;
    let size = i64::from_le_bytes(size);
    Ok(size)
}

fn load_script() -> io::Result<Vec<u8>> {
    let size = get_size()?;
    let mut buffer = Vec::<u8>::new();
    let file = open_file()?;
    let mut bf = BufReader::new(file);
    bf.seek(io::SeekFrom::End(-(size + INDEX_SIZE as i64)))?;
    bf.read_to_end(&mut buffer)?;
    buffer.truncate(buffer.len() - INDEX_SIZE);
    Ok(buffer)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let script = load_script()?;
    // See https://github.com/amethyst/rlua/issues/264
    let lua = unsafe {
        rlua::Lua::unsafe_new_with_flags(
            StdLib::ALL_NO_DEBUG,
            InitFlags::DEFAULT - InitFlags::REMOVE_LOADLIB,
        )
    };
    lua.context(|ctx| -> Result<()> {
        let _g = ctx.globals();
        _g.set("argv", args)?;
        ctx.load(&script).exec()?;
        Ok(())
    })
    .unwrap();
    Ok(())
}
