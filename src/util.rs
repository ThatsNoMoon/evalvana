// Copyright 2020 Benjamin Scherer
// Licensed under the Open Software License version 3.0

use crossbeam_channel::{unbounded as unbounded_channel, Receiver, Sender};

use std::{
	fmt::Debug,
	mem::ManuallyDrop,
	ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Id<T: Debug> {
	inner: ManuallyDrop<T>,
	id_relinquisher: Sender<T>,
}

impl<T: Debug> Id<T> {
	fn new(inner: T, id_relinquisher: Sender<T>) -> Self {
		Id {
			inner: ManuallyDrop::new(inner),
			id_relinquisher,
		}
	}
}

impl<T: PartialEq + Debug> PartialEq for Id<T> {
	fn eq(&self, other: &Id<T>) -> bool {
		self.inner == other.inner
			&& self.id_relinquisher.same_channel(&other.id_relinquisher)
	}
}

impl<T: Eq + Debug> Eq for Id<T> {}

impl<T: Debug> Drop for Id<T> {
	fn drop(&mut self) {
		let id = unsafe { ManuallyDrop::take(&mut self.inner) };
		self.id_relinquisher.try_send(id).unwrap_or_else(|err| {
			if log::log_enabled!(log::Level::Debug) {
				let msg = format!("{}", err);
				log::debug!(
					"Failed to relinquish Id {:?}: {}",
					err.into_inner(),
					msg
				);
			}
		});
	}
}

impl<T: Debug> Deref for Id<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&*self.inner
	}
}

impl<T: Debug> DerefMut for Id<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.inner
	}
}

#[derive(Debug)]
pub struct IdManager<T> {
	id_reclaimer: Receiver<T>,
	id_relinquisher: Sender<T>,
}

impl<T> Default for IdManager<T> {
	fn default() -> Self {
		let (id_relinquisher, id_reclaimer) = unbounded_channel();
		IdManager {
			id_relinquisher,
			id_reclaimer,
		}
	}
}

impl<T: Debug> IdManager<T> {
	pub fn create_id(&mut self, id: T) -> Id<T> {
		let id_relinquisher = self.id_relinquisher.clone();
		Id::new(id, id_relinquisher)
	}
	pub fn reclaimed_ids(&mut self) -> impl Iterator<Item = T> + '_ {
		std::iter::successors(self.id_reclaimer.try_recv().ok(), move |_| {
			self.id_reclaimer.try_recv().ok()
		})
	}
}
