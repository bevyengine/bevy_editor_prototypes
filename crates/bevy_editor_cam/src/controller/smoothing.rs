//! Implements a motion-conserving smoothed input queue.

use std::{
    collections::VecDeque,
    ops::{Add, AddAssign, Mul},
    time::Duration,
};

use bevy::reflect::prelude::*;
use bevy::utils::Instant;
use bevy_derive::{Deref, DerefMut};

/// How smooth should inputs be? Over what tine window should they be averaged.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct Smoothing {
    /// Smoothing window for panning.
    pub pan: Duration,
    /// Smoothing window for orbit.
    pub orbit: Duration,
    /// Smoothing window for zoom.
    pub zoom: Duration,
}

impl Default for Smoothing {
    fn default() -> Self {
        Smoothing {
            pan: Duration::from_millis(10),
            orbit: Duration::from_millis(30),
            zoom: Duration::from_millis(60),
        }
    }
}

/// A smoothed queue of inputs over time.
///
/// Useful for smoothing to query "what was the average input over the last N milliseconds?". This
/// does some important bookkeeping to ensure samples are not over or under sampled. This means the
/// queue has very useful properties:
///
/// 1. The smoothing can change over time, useful for sampling over changing framerates.
/// 2. The sum of smoothed and unsmoothed inputs will be equal despite (1). This is useful because
///    you can smooth something like pointer motions, and the smoothed output will arrive at the
///    same destination as the unsmoothed input without drifting.
#[derive(Debug, Clone, Reflect, Deref, DerefMut)]
pub struct InputQueue<T>(pub VecDeque<InputStreamEntry<T>>);

/// Represents a single input in an [`InputQueue`].
#[derive(Debug, Clone, Reflect)]
pub struct InputStreamEntry<T> {
    /// The time the sample was added and smoothed value computed.
    time: Instant,
    /// The input sample recorded at this time.
    sample: T,
    /// How much of this entry is available to be consumed, from `0.0` to `1.0`. This is required to
    /// ensure that smoothing does not over or under sample any entries as the size of the sampling
    /// window changes. This value should always be zero by the time a sample exits the queue.
    fraction_remaining: f32,
    /// Because we need to do bookkeeping to ensure no samples are under or over sampled, we compute
    /// the smoothed value at the same time a sample is inserted. Because consumers of this will
    /// want to read the smoothed samples multiple times, we do the computation eagerly so the input
    /// stream is always in a valid state, and the act of a user reading a sample multiple times
    /// does not change the value they get.
    smoothed_value: T,
}

impl<T: Copy + Default + Add<Output = T> + AddAssign<T> + Mul<f32, Output = T>> Default
    for InputQueue<T>
{
    fn default() -> Self {
        let start = Instant::now();
        let interval = Duration::from_secs_f32(1.0 / 60.0);
        let mut queue = VecDeque::default();
        for time in
            // See: https://github.com/aevyrie/bevy_editor_cam/issues/13 There is no guarantee that
            // `start` is large enough to subtract from, so we ignore any subtractions that fail, to
            // avoid a panic. If this fails, it will manifest as a slight stutter, most noticeable
            // during zooming. However, this *should* only happen at the very startup of the app,
            // and even then, is unlikely.
            (1..Self::MAX_EVENTS)
                .filter_map(|i| start.checked_sub(interval.mul_f32(i as f32)))
        {
            queue.push_back(InputStreamEntry {
                time,
                sample: T::default(),
                fraction_remaining: 1.0,
                smoothed_value: T::default(),
            });
        }
        Self(queue)
    }
}

impl<T: Copy + Default + Add<Output = T> + AddAssign<T> + Mul<f32, Output = T>> InputQueue<T> {
    const MAX_EVENTS: usize = 256;

    /// Add an input sample to the queue, and compute the smoothed value.
    ///
    /// The smoothing must be computed at the time a sample is added to ensure no samples are over
    /// or under sampled in the smoothing process.
    pub fn process_input(&mut self, new_input: T, smoothing: Duration) {
        let now = Instant::now();
        let queue = &mut self.0;

        // Compute the expected sampling window end index
        let window_size = queue
            .iter()
            .enumerate()
            .find(|(_i, entry)| now.duration_since(entry.time) > smoothing)
            .map(|(i, _)| i) // `find` breaks *after* we fail, so we don't need to add one
            .unwrap_or(0)
            + 1; // Add one to account for the new sample being added

        let range_end = (window_size - 1).clamp(0, queue.len());

        // Compute the smoothed value by sampling over the desired window
        let target_fraction = 1.0 / window_size as f32;
        let mut smoothed_value = new_input * target_fraction;
        for entry in queue.range_mut(..range_end) {
            // Only consume what is left of a sample, to prevent oversampling
            let this_fraction = entry.fraction_remaining.min(target_fraction);
            smoothed_value += entry.sample * this_fraction;
            entry.fraction_remaining = (entry.fraction_remaining - this_fraction).max(0.0);
        }

        // To prevent under sampling, we also need to look at entries older than the window, and add
        // those to the smoothed value, to catch up. This happens when the window shrinks, or there
        // is a pause in rendering and it needs to catch up.
        for old_entry in queue
            .range_mut(range_end..)
            .filter(|e| e.fraction_remaining > 0.0)
        {
            smoothed_value += old_entry.sample * old_entry.fraction_remaining;
            old_entry.fraction_remaining = 0.0;
        }

        queue.truncate(Self::MAX_EVENTS - 1);
        queue.push_front(InputStreamEntry {
            time: now,
            sample: new_input,
            fraction_remaining: 1.0 - target_fraction,
            smoothed_value,
        });
    }

    /// Get the latest motion-conserving smoothed input value.
    pub fn latest_smoothed(&self) -> Option<T> {
        self.iter_smoothed().next().map(|(_, val)| val)
    }

    /// Iterator over all smoothed samples.
    pub fn iter_smoothed(&self) -> impl Iterator<Item = (Instant, T)> + '_ {
        self.0
            .iter()
            .map(|entry| (entry.time, entry.smoothed_value))
    }

    /// Iterate over the raw samples.
    pub fn iter_unsmoothed(&self) -> impl Iterator<Item = (Instant, T)> + '_ {
        self.0.iter().map(|entry| (entry.time, entry.sample))
    }

    /// Approximate the smoothed average sampled in the `window`.
    pub fn average_smoothed_value(&self, window: Duration) -> T {
        let now = Instant::now();
        let mut count = 0;
        let sum = self
            .iter_smoothed()
            .filter(|(t, _)| now.duration_since(*t) < window)
            .map(|(_, smoothed_value)| smoothed_value)
            .reduce(|acc, v| {
                count += 1;
                acc + v
            })
            .unwrap_or_default();
        sum * (1.0 / count as f32)
    }

    /// Approximate smoothed value with user-supplied modifier function as needed
    pub fn approx_smoothed(&self, window: Duration, mut modifier: impl FnMut(&mut T)) -> T {
        let now = Instant::now();
        let n_elements = &mut 0;
        self.iter_unsmoothed()
            .filter(|(time, _)| now.duration_since(*time) < window)
            .map(|(_, value)| {
                *n_elements += 1;
                let mut value = value;
                modifier(&mut value);
                value
            })
            .reduce(|acc, v| acc + v)
            .unwrap_or_default()
            * (1.0 / *n_elements as f32)
    }
}
