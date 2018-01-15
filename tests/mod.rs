extern crate bidule;

use bidule::*;

#[test]
fn test_0() {
  let label_0 = Label::new();
  let mut label_0_ = label_0.clone();
  let mut label = Label::new();

  label.subscribe(move |signal| {
    match *signal {
      LabelSignal::TextChanged(ref txt) => label_0_.set(txt),
      LabelSignal::Submit => label_0_.reset()
    }
  });

  label.set("coucou");
  assert_eq!(label_0.borrow().as_str(), "coucou");

  label.submit();
  assert_eq!(label_0.borrow().as_str(), "");
}

#[test]
fn test_1() {
  let label = Label::new();
  let mut label_0 = label.clone();
  let mut button = ToggleButton::new();

  button.subscribe(move |signal| {
    match *signal {
      ToggleButtonSignal::Pressed => label_0.set("hey!"),
      ToggleButtonSignal::Released => label_0.reset()
    }
  });

  assert_eq!(label.borrow().as_str(), "");

  button.push();
  assert_eq!(label.borrow().as_str(), "hey!");
  
  button.release();
  assert_eq!(label.borrow().as_str(), "");
}
