use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub trait Bidule {
  // Type of signal this `Bidule` can emit.
  type Signal;

  // React to an external event.
  //fn react<E, S>(&mut self, event: &E, app_st: &mut S);

  // Add a subscriber to listen to signals coming out of this `Bidule`.
  fn subscribe<F>(&mut self, subscriber: F) where F: 'static + FnMut(&Self::Signal);
}

pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T> {
  pub fn new(x: T) -> Self {
    Shared(Rc::new(RefCell::new(x)))
  }
}

impl<T> Clone for Shared<T> {
  fn clone(&self) -> Self {
    Shared(self.0.clone())
  }
}

impl<T> Deref for Shared<T> {
  type Target = Rc<RefCell<T>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

// Text label.
pub struct Label {
  text: String,
  subscribers: Vec<Box<FnMut(&LabelSignal)>>
}

impl Label {
  pub fn new() -> Shared<Self> {
    Self::with_text("")
  }

  pub fn with_text(text: &str) -> Shared<Self> {
    let label =
      Label {
        text: text.to_owned(),
        subscribers: Vec::new()
      };

    Shared::new(label)
  }

  fn notify(&mut self, signal: LabelSignal) {
    for sub in &mut self.subscribers {
      sub(&signal);
    }
  }

  pub fn as_str(&self) -> &str {
    &self.text
  }
}

impl Shared<Label> {
  pub fn set(&mut self, text: &str) {
    let mut self_ = self.borrow_mut();

    self_.text = text.to_owned();
    self_.notify(LabelSignal::TextChanged(text.to_owned()));
  }

  pub fn reset(&mut self) {
    self.set("")
  }

  pub fn submit(&mut self) {
    self.borrow_mut().notify(LabelSignal::Submit);
  }
}

impl Bidule for Shared<Label> {
  type Signal = LabelSignal;

  fn subscribe<F>(&mut self, subscriber: F) where F: 'static + FnMut(&Self::Signal) {
    self.0.borrow_mut().subscribers.push(Box::new(subscriber));
  }
}

// Signals that can be emited by a `Label`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LabelSignal {
  TextChanged(String),
  Submit
}

// On/off button.
pub struct ToggleButton {
  pushed: bool,
  subscribers: Vec<Box<FnMut(&ToggleButtonSignal)>>
}

impl ToggleButton {
  pub fn new() -> Shared<Self> {
    Self::with_pushed(false)
  }

  pub fn with_pushed(pushed: bool) -> Shared<Self> {
    let button =
      ToggleButton {
        pushed,
        subscribers: Vec::new()
      };

    Shared::new(button)
  }

  fn notify(&mut self, signal: ToggleButtonSignal) {
    for sub in &mut self.subscribers {
      sub(&signal);
    }
  }
}

impl Shared<ToggleButton> {
  pub fn push(&mut self) {
    let mut self_ = self.0.borrow_mut();

    self_.pushed = true;
    self_.notify(ToggleButtonSignal::Pressed);
  }

  pub fn release(&mut self) {
    let mut self_ = self.0.borrow_mut();

    self_.pushed = false;
    self_.notify(ToggleButtonSignal::Released);
  }

  pub fn toggle(&mut self) {
    let mut self_ = self.0.borrow_mut();

    if self_.pushed {
      self_.pushed = false;
      self_.notify(ToggleButtonSignal::Released);
    } else {
      self_.pushed = true;
      self_.notify(ToggleButtonSignal::Pressed);
    }
  }
}

impl Bidule for Shared<ToggleButton> {
  type Signal = ToggleButtonSignal;

  fn subscribe<F>(&mut self, subscriber: F) where F: 'static + FnMut(&Self::Signal) {
    self.0.borrow_mut().subscribers.push(Box::new(subscriber));
  }
}

// Signals that can be emited by a `ToggleButton`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToggleButtonSignal {
  Pressed,
  Released
}
