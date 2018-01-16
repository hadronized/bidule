use std::cell::RefCell;
use std::rc::{Rc, Weak};

type Subscribers<Sig> = RefCell<Vec<Box<FnMut(&Sig)>>>;

enum SubscribersRef<Sig> {
  Own(Rc<Subscribers<Sig>>),
  Weak(Weak<Subscribers<Sig>>)
}

/// A stream of signals.
///
/// A stream represents a composable signal producer. When you decide to send a signal down a
/// stream, any other streams composed with that first stream will also receive the signal. This
/// enables to construct more interesting and complex streams by composing them.
pub struct Stream<Sig> {
  subscribers: SubscribersRef<Sig>
}

impl<Sig> Stream<Sig> where Sig: 'static {
  /// Create a new stream.
  pub fn new() -> Self {
    let subscribers = SubscribersRef::Own(Rc::new(RefCell::new(Vec::new())));
    Stream { subscribers }
  }

  /// Create a new, non-owned, version of this stream.
  fn new_weak(&self) -> Self {
    let subscribers = match self.subscribers {
      SubscribersRef::Own(ref rc) => SubscribersRef::Own(rc.clone()),
      SubscribersRef::Weak(ref weak) => SubscribersRef::Weak(weak.clone())
    };

    Stream { subscribers }
  }

  /// Subscribe a new listener for this stream’s signals.
  ///
  /// This function enables to “observe” any signal flowing out of the stream. However, do not abuse
  /// this function, as its primary use is to build other combinators.
  pub fn subscribe<F>(&self, subscriber: F) where F: 'static + FnMut(&Sig) {
    match self.subscribers {
      SubscribersRef::Own(ref subscribers) => subscribers.borrow_mut().push(Box::new(subscriber)),
      SubscribersRef::Weak(ref weak) => {
        if let Some(subscribers) = weak.upgrade() {
          subscribers.borrow_mut().push(Box::new(subscriber));
        }
      }
    }
  }

  /// Send a signal down the stream.
  pub fn send(&self, signal: &Sig) {
    match self.subscribers {
      SubscribersRef::Own(ref subscribers) => {
        for sub in subscribers.borrow_mut().iter_mut() {
          sub(signal);
        }
      }

      SubscribersRef::Weak(ref weak) => {
        if let Some(subscribers) = weak.upgrade() {
          for sub in subscribers.borrow_mut().iter_mut() {
            sub(signal);
          }
        }
      }
    }
  }

  /// Map any signals flowing out a stream.
  ///
  /// Please note that this function is total: you cannot ignore signals. Even if you map
  /// *uninteresting signals* to `None`, you’ll still compose signals for those. If are interested
  /// in filtering signals while mapping, have a look at the `filter_map` function.
  pub fn map<F, OutSig>(
    &self,
    f: F
  ) -> Stream<OutSig>
    where F: 'static + Fn(&Sig) -> OutSig,
          OutSig: 'static {
    let mapped_stream = Stream::new();
    let mapped_stream_ = mapped_stream.new_weak();

    self.subscribe(move |sig| {
      mapped_stream_.send(&f(sig));
    });

    mapped_stream
  }

  /// Filter and map signals flowing out a stream.
  ///
  /// If you’re not interested in a specific signal, you can emit `None`: no signal will be sent.
  pub fn filter_map<F, OutSig>(
    &self,
    f: F
  ) -> Stream<OutSig>
    where F: 'static + Fn(&Sig) -> Option<OutSig>,
          OutSig: 'static {
    let mapped_stream = Stream::new();
    let mapped_stream_ = mapped_stream.new_weak();

    self.subscribe(move |sig| {
      if let Some(ref mapped_sig) = f(sig) {
        mapped_stream_.send(mapped_sig);
      }
    });

    mapped_stream
  }

  /// Filter the signals flowing out of a stream with a predicate.
  pub fn filter<F>(&self, pred: F) -> Self where F: 'static + Fn(&Sig) -> bool {
    let filtered = Stream::new();
    let filtered_ = filtered.new_weak();

    self.subscribe(move |sig| {
      if pred(sig) {
        filtered_.send(sig);
      }
    });

    filtered
  }

  /// Fold all signals flowing out of a stream into a stream of values.
  pub fn fold<F, A>(
    &self,
    value: A,
    f: F
  ) -> Stream<A>
    where F: 'static + Fn(A, &Sig) -> A,
          A: 'static {
    let folded_stream = Stream::new();
    let folded_stream_ = folded_stream.new_weak();
    let mut boxed = Some(value);

    self.subscribe(move |sig| {
      if let Some(value) = boxed.take() {
        let output = f(value, sig);
        folded_stream_.send(&output);
        boxed = Some(output);
      }
    });

    folded_stream
  }

  /// Merge two streams into one.
  ///
  /// Merging streams enables you to perform later useful compositions, such as folding the merged
  /// results.
  pub fn merge(&self, rhs: &Self) -> Self {
    let merged = Stream::new();
    let merged_self = merged.new_weak();
    let merged_rhs = merged.new_weak();

    self.subscribe(move |sig| {
      merged_self.send(sig);
    });

    rhs.subscribe(move |sig| {
      merged_rhs.send(sig);
    });

    merged
  }

  /// Create a pair of entangled streams.
  ///
  /// If any of the streams sends a signal, the other one receives it. However, be careful: since
  /// the signals are defined in terms of each other, it’s quite easy to cause infinite loops if you
  /// don’t have a well-defined bottom to your recursion. This is why you’re expected to return
  /// `Option<_>` signals.
  pub fn entangled<F, G, GSig>(
    f: F,
    g: G
  ) -> (Self, Stream<GSig>)
    where F: 'static + Fn(&Sig) -> Option<GSig>,
          G: 'static + Fn(&GSig) -> Option<Sig>,
          GSig: 'static {
    let fs = Stream::new();
    let gs = Stream::new();
    let fs_ = fs.new_weak();
    let gs_ = gs.new_weak();

    fs.subscribe(move |sig| {
      if let Some(sig_) = f(sig) {
        gs_.send(&sig_);
      }
    });

    gs.subscribe(move |sig| {
      if let Some(sig_) = g(sig) {
        fs_.send(&sig_);
      }
    });

    (fs, gs)
  }
}

impl<SigA, SigB> Stream<(SigA, SigB)> where SigA: 'static, SigB: 'static {
  /// Split a stream of merged values into two streams.
  pub fn unmerge(&self) -> (Stream<SigA>, Stream<SigB>) {
    let a = Stream::new();
    let a_ = a.new_weak();
    let b = Stream::new();
    let b_ = b.new_weak();

    self.subscribe(move |sig| {
      a_.send(&sig.0);
      b_.send(&sig.1);
    });

    (a, b)
  }
}
