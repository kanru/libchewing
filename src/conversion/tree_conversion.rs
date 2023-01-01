use std::{cell::RefCell, rc::Rc};

use crate::dictionary::Dictionary;

use super::{ConversionEngine, ChineseSequence, Interval};

#[derive(Debug)]
pub struct TreeConversionEngine {
    dict: Rc<RefCell<dyn Dictionary>>,
}

impl ConversionEngine for TreeConversionEngine {
    fn convert(&self, segment: &ChineseSequence) -> Vec<Interval> {
        todo!()
    }

    fn convert_next(&self, segment: &ChineseSequence, next: usize) -> Vec<Interval> {
        todo!()
    }
}
