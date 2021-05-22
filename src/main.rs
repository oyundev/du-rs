#[macro_use]
extern crate may;
use may::coroutine;
use std::path::PathBuf;
use rustop::opts;
use std::sync::atomic::{AtomicU64, Ordering};
// https://github.com/Xudong-Huang/may

fn main() {    
    let (args, _rest) = opts! {
        synopsis "Calculate total directory size (MB) recursively.";
		 opt poolsize:usize=1024,
				desc:"Set pool capacity."; 
		 opt stacksize:usize=3072,
				desc:"Set stack size."; 
		 opt threads:usize=4,
				desc:"Number of worker threads."; 
        param folder:Option<String>, desc:"Target directory name.";
    }.parse_or_exit();
	 
	may::config().set_pool_capacity(args.poolsize);
	may::config().set_stack_size(args.stacksize);
	may::config().set_workers(args.threads);
	let mut startfolder = String::from(".");	
	if let Some(folder) = args.folder { startfolder = folder; }
	let n = disk_usage(PathBuf::from(startfolder));
	println!("{:.2}M", (n as f64) / 1048576_f64); 
}

fn disk_usage(currpath: PathBuf) -> u64 {
    let meta = std::fs::metadata(&currpath).unwrap();
    let file_type = meta.file_type();
    if file_type.is_dir() {
        let total = AtomicU64::new(0);
        coroutine::scope(|scope| {
            for entry in std::fs::read_dir(&currpath).unwrap() {
                let e = entry.unwrap();
                let p = e.path();
                let m = e.metadata().unwrap(); 
                if m.file_type().is_dir() { 
                    go!(scope, || total.fetch_add(disk_usage(p),
                        Ordering::Relaxed
                    ));
                } else {
                    total.fetch_add(
                        m.len(),
                        Ordering::Relaxed
                    );
                }
            }
        });
        return total.load(Ordering::Relaxed);
    } else if file_type.is_file() {
        return meta.len();
    } else {
        return 0;
    }
}
