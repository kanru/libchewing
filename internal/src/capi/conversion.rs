use std::{
    cell::RefCell,
    ffi::{c_char, c_int, c_void, CStr, CString},
    rc::Rc,
    slice,
};

use chewing::{
    conversion::{Break, ChewingConversionEngine, ChineseSequence, ConversionEngine, Interval},
    dictionary::LayeredDictionary,
};

/// cbindgen:ignore
#[repr(C)]
pub struct IntervalType {
    from: c_int,
    to: c_int,
}

#[no_mangle]
pub extern "C" fn InitConversionEngine(
    dict_ptr: *const RefCell<LayeredDictionary>,
) -> *mut ChewingConversionEngine {
    let dict = unsafe { Rc::from_raw(dict_ptr) };
    let engine = Box::new(ChewingConversionEngine::new(dict.clone()));
    Rc::into_raw(dict);
    Box::into_raw(engine)
}

#[no_mangle]
pub extern "C" fn TerminateConversionEngine(ce_ptr: *mut ChewingConversionEngine) {
    if ce_ptr.is_null() {
        return;
    }
    unsafe { Box::from_raw(ce_ptr) };
}

#[no_mangle]
pub extern "C" fn ConversionEngineDoPhrasing(
    pgdata: *mut c_void,
    ce_ptr: *mut ChewingConversionEngine,
    syllables_u16_ptr: *mut u16,
    syllables_len: usize,
    select_strs_ptr: *mut [c_char; 201],
    select_intervals_ptr: *mut IntervalType,
    select_len: usize,
    breaks_u16_ptr: *mut c_int,
    breaks_len: usize,
    display_intervals_ptr: *mut IntervalType,
    display_intervals_len: *mut c_int,
) {
    let ce = unsafe { ce_ptr.as_ref().expect("nonnull pointer") };

    let syllables_u16 = unsafe { slice::from_raw_parts(syllables_u16_ptr, syllables_len) };
    let syllables = syllables_u16
        .iter()
        .map(|&syl_u16| syl_u16.try_into().expect("convert u16 to syllable"))
        .collect();

    let select_strs: Vec<_> = unsafe { slice::from_raw_parts(select_strs_ptr, select_len) }
        .iter()
        .map(|it| unsafe { CStr::from_ptr(it.as_ptr()) })
        .collect();
    let select_intervals: Vec<_> =
        unsafe { slice::from_raw_parts(select_intervals_ptr, select_len) }
            .iter()
            .collect();
    let selections = select_intervals
        .into_iter()
        .zip(select_strs.into_iter())
        .map(|(interval, str)| Interval {
            start: interval.from as usize,
            end: interval.to as usize,
            phrase: str.to_string_lossy().to_string(),
        })
        .collect();

    let mut breaks = vec![];
    unsafe { slice::from_raw_parts(breaks_u16_ptr, breaks_len) }
        .iter()
        .enumerate()
        .for_each(|(i, &br)| {
            if br == 1 {
                breaks.push(Break(i));
            }
        });

    let sequence = ChineseSequence {
        syllables,
        selections,
        breaks,
    };
    let intervals = ce.convert_next(&sequence, 0);

    let display_intervals =
        unsafe { slice::from_raw_parts_mut(display_intervals_ptr, intervals.len()) };
    unsafe {
        *display_intervals_len = intervals.len() as c_int;
    }

    for (i, interval) in intervals.into_iter().enumerate() {
        let from = interval.start as c_int;
        let to = interval.end as c_int;
        let phrase = CString::new(interval.phrase).expect("phrase has no internal null");
        unsafe {
            crate::binding::FillPreeditBuf(pgdata.cast(), phrase.as_ptr() as *mut i8, from, to);
        }
        display_intervals[i].from = from;
        display_intervals[i].to = to;
    }
}
