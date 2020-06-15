use crossbeam_channel::{unbounded as unbounded_channel, Receiver, Sender};
use tinyvec::TinyVec;

use std::{fmt::Display, mem::ManuallyDrop};

#[derive(Debug)]
pub struct Id<T: Display + Copy> {
	pub inner: T,
	id_relinquisher: Sender<T>,
}

impl<T: PartialEq + Display + Copy> PartialEq for Id<T> {
	fn eq(&self, other: &Id<T>) -> bool {
		self.inner == other.inner
			&& self.id_relinquisher.same_channel(&other.id_relinquisher)
	}
}

impl<T: PartialEq + Display + Copy> Eq for Id<T> {}

impl<T: Display + Copy> Drop for Id<T> {
	fn drop(&mut self) {
		self.id_relinquisher
			.try_send(self.inner)
			.unwrap_or_else(|err| {
				log::debug!(
					"Failed to relinquish DrawingId {}: {}",
					err.into_inner(),
					err,
				)
			});
	}
}

#[derive(Debug)]
pub struct IdManager<T> {
	id_reclaimer: Receiver<T>,
	id_relinquisher: Sender<T>,
}

impl<T: Display + Copy> Default for IdManager<T> {
	fn default() -> Self {
		let (id_relinquisher, id_reclaimer) = unbounded_channel();
		IdManager {
			id_relinquisher,
			id_reclaimer,
		}
	}
}

impl<T: Display + Copy> IdManager<T> {
	pub fn create_id(&mut self, id: T) -> Id<T> {
		let id_relinquisher = self.id_relinquisher.clone();
		Id {
			inner: id,
			id_relinquisher,
		}
	}
	pub fn reclaimed_ids(&mut self) -> impl Iterator<Item = T> + '_ {
		std::iter::successors(self.id_reclaimer.try_recv().ok(), move |_| {
			self.id_reclaimer.try_recv().ok()
		})
	}
}
