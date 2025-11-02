use biquad::{Biquad, DirectForm1};
use rodio::Source;
use std::sync::{Arc, Mutex};

pub struct FilteredSource<S> {
    pub(crate) source: S,
    pub(crate) filter: Arc<Mutex<DirectForm1<f32>>>,
}

impl<S> Iterator for FilteredSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.source.next()?;
        let mut filter = self.filter.lock().unwrap();
        Some(filter.run(sample))
    }
}

impl<S> Source for FilteredSource<S>
where
    S: Source<Item = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }
    fn channels(&self) -> u16 {
        self.source.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.source.total_duration()
    }
}
