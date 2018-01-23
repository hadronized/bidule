//! The bidule FRP crate.
//!
//! This crate provides a few simple primitives to write FRP-driven programs. Everything revolves
//! around the concept of a `Stream`.
//!
//! # Streams
//!
//! A `Stream` is a *stream of typed signals*. A stream of signals will get a *signal* as input
//! and will broadcast it downwards. You can compose streams with each other with very simple
//! combinators, such as `map`, `filter`, `filter_map`, `zip`, `unzip`, `merge`, `fold`, `sink`,
//! etc.
//!
//! ## Creating streams and send signals
//!
//! Streams are typed. You can use type inference or give them an explicit type:
//!
//! ```
//! use bidule::Stream;
//!
//! let my_stream: Stream<i32> = Stream::new();
//! ```
//!
//! That’s all you need to create a stream. A stream represent a value that will be flowing in *at
//! some time*.
//!
//! > Even though it’s not strictly the same thing, you can see a similitude with
//! > [futures](https://crates.io/crates/futures).
//!
//! When you’re ready to send signals, just call the `send` function:
//!
//! ```
//! use bidule::Stream;
//!
//! let my_stream: Stream<i32> = Stream::new();
//!
//! my_stream.send(&1);
//! my_stream.send(&2);
//! my_stream.send(&3);
//! ```
//!
//! ## Subscriptions
//!
//! A single stream like that one won’t do much – actually, it’ll do nothing. The first thing we
//! might want to do is to subscribe a closure to do something when a signal is emitted. This is
//! done with the `subscribe` function.
//!
//! ```
//! use bidule::Stream;
//!
//! let my_stream: Stream<i32> = Stream::new();
//! 
//! my_stream.subscribe(|sig| {
//!   // print the signal on stdout each time it’s flowing in
//!   println!("signal: {:?}", sig);
//! });
//!
//! my_stream.send(&1);
//! my_stream.send(&2);
//! my_stream.send(&3);
//! ```
//!
//! ## FRP basics
//!
//! However, FRP is not about callbacks. It’s actually the opposite, to be honest. We try to reduce
//! the use of callbacks as much as possible. FRP solves this by inversing the way you must work:
//! instead of subscribing callbacks to react to something, you transform that something to create
//! new values or objects. This is akin to the kind of transformations you do with `Future`.
//!
//! Let’s get our feet wet: let’s create a new stream that will only emit signals for even values:
//!
//! ```
//! use bidule::Stream;
//!
//! let int_stream: Stream<i32> = Stream::new();
//! let even_stream = int_stream.filter(|x| x % 2 == 0);
//! ```
//!
//! `even_stream` has type `Stream<i32>` and will only emit signals when the input signal is `even`.
//!
//! Let’s try something more complicated: on those signals, if the value is less or equal to 10,
//! output `"Hello, world!"`; otherwise, output `"See you!"`.
//!
//! ```
//! use bidule::Stream;
//!
//! let int_stream: Stream<i32> = Stream::new();
//! let even_stream = int_stream.filter(|x| x % 2 == 0);
//! let str_stream = even_stream.map(|x| if *x <= 10 { "Hello, world!" } else { "See you!" });
//! ```
//!
//! This is really easy; no trap involved.
//!
//! Ok, let’s try something else. Some kind of a *Hello world* for FRP.
//!
//! ```
//! use bidule::Stream;
//!
//! enum Button {
//!   Pressed,
//!   Released
//! }
//!
//! fn unbuttonify(button: &Button, v: i32) -> Option<i32> {
//!   match button {
//!     ref Released => Some(v),
//!     _ => None
//!   }
//! }
//!
//! let minus = Stream::new();
//! let plus = Stream::new();
//! let counter =
//!   minus.filter_map(|b| unbuttonify(b, -1))
//!        .merge(&plus.filter_map(|b| unbuttonify(b, 1)))
//!        .fold(0, |a, x| a + x);
//! ```
//!
//! In this snippet, we have two buttons: `minus` and `plus`. If we hit the `minus` button, we want
//! a counter to be decremented and if we hit the `plus` button, the counter must increment.
//!
//! FRP solves that problem by expressing `counter` in terms of both `minus` and `plus`. The first
//! thing we do is to map a number on the stream that broadcasts button signals. Whenever that
//! signal is a `Button::Released`, we return a given number. For `minus`, we return `-1` and for
//! `plus`, we return `1` – or `+1`, it’s the same thing. That gives us two new streams. Let’s see
//! the types to have a deeper understanding:
//!
//! - `minus: Stream<Button>`
//! - `plus: Stream<Button>`
//! - `minus.filter_map(|b| unbuttonify(b, -1)): Stream<i32>`
//! - `plus.filter_map(|b| unbuttonify(b, 1)): Stream<i32>`
//!
//! The `merge` method is very simple: it takes two streams that emit the same type of signals and
//! merges them into a single stream that will broadcasts both the signals:
//!
//! - `minus.filter_map(|b| unbuttonify(b, -1)).merge(plus.filter_map(|b| unbuttonify(b, 1)): Stream<i32>): Stream<i32>`
//!
//! The next and final step is to `fold` those `i32` into the final value of the counter by applying
//! successive additions. This is done with the `fold` method, that takes the initial value – in the
//! case of a counter, it’s `0` – and the function to accumulate, with the accumulator as first
//! argument and the iterated value as second argument.
//!
//! The resulting stream, which type is `Stream<i32>`, will then contain the value of the counter.
//! You can test it by sending `Button` signals on both `minus` and `plus`: the resulting signal
//! in `counter` will be correctly decrementing or incrementing.
//!
//! There exist several more, interesting combinators to work with your streams. For instance, if
//! you don’t want to map a function over two streams to make them compatible with each other – they
//! have different types, you can still perform some kind of a merge. That operation is called a
//! `zip` and the resulting stream will yield either the value from the first – left – stream or the
//! value of the other – right – as soon as a signal is emitted. Its dual method is called `unzip`
//! and will split a stream apart into two streams if it’s a zipped stream. See `Either` for further
//! details.
//!
//! ## Sinking
//!
//! *Sinking* is the action to consume the signals of a stream and collect them. The current
//! implementation uses non-blocking buffering when sending signals, and reading is up to you: the
//! stream will collect the output signals in a buffer you can read via the
//! [Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html) trait. For instance,
//! non-blocking reads:
//!
//! ```
//! use bidule::Stream;
//!
//! enum Button {
//!   Pressed,
//!   Released
//! }
//!
//! fn unbuttonify(button: &Button, v: i32) -> Option<i32> {
//!   match button {
//!     ref Released => Some(v),
//!     _ => None
//!   }
//! }
//!
//! let minus = Stream::new();
//! let plus = Stream::new();
//! let counter =
//!   minus.filter_map(|b| unbuttonify(b, -1))
//!        .merge(&plus.filter_map(|b| unbuttonify(b, 1)))
//!        .fold(0, |a, x| a + x);
//!
//! let rx = counter.sink();
//!
//! // do something with minus and plus
//! // …
//!
//! for v in rx.try_iter() {
//!   println!("read a new value of the counter: {}", v);
//! }
//! ```

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{Receiver, channel};

type Subscribers<'a, Sig> = RefCell<Vec<Box<FnMut(&Sig) + 'a>>>;

enum SubscribersRef<'a, Sig> {
  Own(Rc<Subscribers<'a, Sig>>),
  Weak(Weak<Subscribers<'a, Sig>>)
}

/// Either one or another type.
///
/// This type is especially useful for zipping and unzipping streams. If a stream has a type like
/// `Stream<Either<A, B>>`, it means you can unzip it and get two streams: `Stream<A>` and
/// `Stream<B>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Either<A, B> {
  Left(A),
  Right(B)
}

/// A stream of signals.
///
/// A stream represents a composable signal producer. When you decide to send a signal down a
/// stream, any other streams composed with that first stream will also receive the signal. This
/// enables to construct more interesting and complex streams by composing them.
pub struct Stream<'a, Sig> {
  subscribers: SubscribersRef<'a, Sig>
}

impl<'a, Sig> Stream<'a, Sig> where Sig: 'a {
  /// Create a new stream.
  pub fn new() -> Self {
    let subscribers = SubscribersRef::Own(Rc::new(RefCell::new(Vec::new())));
    Stream { subscribers }
  }

  /// Create a new, version of this stream by behaving the same way as the input reference (if it’s
  /// an owned pointer, it clones ownership; if it’s a weak pointer, it clone the weak pointer).
  fn new_same(&self) -> Self {
    let subscribers = match self.subscribers {
      SubscribersRef::Own(ref rc) => SubscribersRef::Own(rc.clone()),
      SubscribersRef::Weak(ref weak) => SubscribersRef::Weak(weak.clone())
    };

    Stream { subscribers }
  }

  /// Create new, non-owning version of this stream.
  fn new_weak(&self) -> Self {
    let subscribers = match self.subscribers {
      SubscribersRef::Own(ref rc) => SubscribersRef::Weak(Rc::downgrade(rc)),
      SubscribersRef::Weak(ref weak) => SubscribersRef::Weak(weak.clone())
    };

    Stream { subscribers }
  }

  /// Subscribe a new listener for this stream’s signals.
  ///
  /// This function enables to “observe” any signal flowing out of the stream. However, do not abuse
  /// this function, as its primary use is to build other combinators.
  pub fn subscribe<F>(&self, subscriber: F) where F: 'a + FnMut(&Sig) {
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
  ) -> Stream<'a, OutSig>
    where F: 'a + Fn(&Sig) -> OutSig,
          OutSig: 'a {
    let mapped_stream = Stream::new();
    let mapped_stream_ = mapped_stream.new_same();

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
  ) -> Stream<'a, OutSig>
    where F: 'a + Fn(&Sig) -> Option<OutSig>,
          OutSig: 'a {
    let mapped_stream = Stream::new();
    let mapped_stream_ = mapped_stream.new_same();

    self.subscribe(move |sig| {
      if let Some(ref mapped_sig) = f(sig) {
        mapped_stream_.send(mapped_sig);
      }
    });

    mapped_stream
  }

  /// Filter the signals flowing out of a stream with a predicate.
  pub fn filter<F>(&self, pred: F) -> Self where F: 'a + Fn(&Sig) -> bool {
    let filtered = Stream::new();
    let filtered_ = filtered.new_same();

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
  ) -> Stream<'a, A>
    where F: 'a + Fn(A, &Sig) -> A,
          A: 'a {
    let folded_stream = Stream::new();
    let folded_stream_ = folded_stream.new_same();
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
    let merged_self = merged.new_same();
    let merged_rhs = merged.new_same();

    self.subscribe(move |sig| {
      merged_self.send(sig);
    });

    rhs.subscribe(move |sig| {
      merged_rhs.send(sig);
    });

    merged
  }

  // FIXME: see whether we can do the same thing without Clone
  /// Zip two streams with each other.
  pub fn zip<SigRHS>(
    &self,
    rhs: &Stream<'a, SigRHS>
  ) -> Stream<'a, Either<Sig, SigRHS>>
  where Sig: Clone,
        SigRHS: 'a + Clone {
    let zipped = Stream::new();
    let zipped_self = zipped.new_same();
    let zipped_rhs = zipped.new_same();

    self.subscribe(move |sig| {
      zipped_self.send(&Either::Left(sig.clone()));
    });

    rhs.subscribe(move |sig| {
      zipped_rhs.send(&Either::Right(sig.clone()));
    });

    zipped
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
  ) -> (Self, Stream<'a, GSig>)
    where F: 'a + Fn(&Sig) -> Option<GSig>,
          G: 'a + Fn(&GSig) -> Option<Sig>,
          GSig: 'a {
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

  /// Sink a stream.
  pub fn sink(&self) -> Receiver<Sig> where Sig: Clone {
    let (sx, rx) = channel();

    self.subscribe(move |sig| {
      let _ = sx.send(sig.clone());
    });

    rx
  }
}

impl<'a, SigA, SigB> Stream<'a, Either<SigA, SigB>> where SigA: 'static, SigB: 'static {
  /// Split a stream of zipped values into two streams.
  pub fn unzip(&self) -> (Stream<'a, SigA>, Stream<'a, SigB>) {
    let a = Stream::new();
    let a_ = a.new_same();
    let b = Stream::new();
    let b_ = b.new_same();

    self.subscribe(move |sig| {
      match *sig {
        Either::Left(ref l) => a_.send(l),
        Either::Right(ref r) => b_.send(r)
      }
    });

    (a, b)
  }
}
