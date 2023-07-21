use std::thread;

pub enum JobResult {
    Ok,
}

pub struct CompileJobOptions {
    pub source: String,
    pub destination: Option<String>,
}

pub struct CompileJob {
    thread: thread::JoinHandle<JobResult>,
}
impl CompileJob {
    fn spawn(options: CompileJobOptions) -> Self {
        let thread = thread::spawn(move || JobResult::Ok);
        CompileJob { thread }
    }
    fn join(self) -> JobResult {
        self.thread.join().unwrap()
    }
}
