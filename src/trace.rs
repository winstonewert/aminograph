pub trait Tracer {
    fn open(&mut self, name: &str);
    fn open_ex<F: FnOnce() -> String>(&mut self, f: F);
    fn close<D: std::fmt::Debug>(&mut self, value: D) -> D;
    fn data<D: std::fmt::Debug>(&self, name: &str, value: D);
}

pub struct NullTracer;

impl Tracer for NullTracer {
    fn data<D: std::fmt::Debug>(&self, _name: &str, _value: D) {}

    fn open(&mut self, _name: &str) {}

    fn close<D: std::fmt::Debug>(&mut self, value: D) -> D {
        value
    }

    fn open_ex<F: FnOnce() -> String>(&mut self, _f: F) {}
}

pub struct BasicTracer {
    prefix: Vec<String>,
    current: Vec<String>,
}

impl BasicTracer {
    pub fn new(prefix: &str) -> Self {
        BasicTracer {
            prefix: prefix
                .split(".")
                .filter(|x| !x.is_empty())
                .map(|x| x.to_owned())
                .collect(),
            current: Vec::new(),
        }
    }

    fn is_active(&self) -> bool {
        self.prefix == self.current
    }
}

impl Tracer for BasicTracer {
    fn data<D: std::fmt::Debug>(&self, name: &str, value: D) {
        if self.is_active() {
            eprintln!("{} => {:?}", name, value);
        }
    }

    fn open(&mut self, name: &str) {
        self.current.push(name.to_owned());
    }

    fn close<D: std::fmt::Debug>(&mut self, value: D) -> D {
        if self.is_active() {
            eprintln!("=> {:?}", value);
        }
        let tail = self.current.pop().unwrap();
        if self.is_active() {
            eprintln!("{} => {:?} [+]", tail, value);
        }
        value
    }

    fn open_ex<F: FnOnce() -> String>(&mut self, f: F) {
        self.open(&f())
    }
}
