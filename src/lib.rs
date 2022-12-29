use std::collections::{TryReserveError, HashSet};
use std::ops::{Index, IndexMut};


#[derive(Clone,Debug)]
pub struct Heap<T> {
	inner: Vec<(T, Option<usize>)>,
	free: HashSet<usize>,
	len: usize,
}

// Assumptions:
// 	Where P is the parent of node C, the index of P < the index of C
// 	The only valid node with parent `None` is the root node
// 	The root node cannot be invalidated or removed
// 	Therefore, the node with the index `0` is the root node
// 	Every node has a valid parent

impl<T: std::fmt::Debug> Heap<T> {
	pub fn new(root: T) -> Self {
		Self {
			inner: vec![(root, None)],
			free: HashSet::from([1]),
			len: 1,
		}
	}
	pub fn with_capacity(capacity: usize, root: T) -> Self {
		let mut inner = Vec::with_capacity(capacity);
		inner.push((root, None));
		Self {
			inner,
			free: HashSet::from([1]),
			len: 1,
		}
	}

	/// Panics if `parent` is not the index of a valid node
	pub fn insert(&mut self, node: T, parent: usize) -> usize {
		assert!(
			self.is_valid_idx(parent),
			"Heap: Error: Tried to insert with invalid parent"
		);
		let i = *self.free.iter().skip_while(|x| x <= &&parent).next()
			.expect("Heap: Internal Error: Missing trailing free index.");
		self.free.remove(&i);
		if i >= self.len() {
			self.free.insert(i + 1);
			self.inner.push((node, Some(parent)));
		} else {
			self[i] = (node, Some(parent));
		}
		self.len += 1;
		i
	}

	/// Panics if `index` is invalid.
	/// Panics if `index` is 0.
	pub fn remove(&mut self, index: usize) {
		assert!(index > 0, "Heap: Error: Tried to remove the root node.");
		assert!(self.is_valid_idx(index), "Heap: Error: Tried to remove an invalid node");
		let mut should_remove: HashSet<_> = HashSet::from([index]);
		should_remove.extend(self.descendants_of(index));
		for node in should_remove {
			self.free.insert(node);
			self[node].1 = None;
			self.len -= 1;
		}
	}

	pub fn descendants_of(&self, index: usize) -> HashSet<usize> {
		let mut descendants = HashSet::from([index]);
		for (idx, node) in self.inner.iter().enumerate().skip(index + 1) {
			if let Some(i) = node.1 {
				if descendants.contains(&i) {
					descendants.insert(idx);
				}
			}
		}
		descendants.remove(&index);
		descendants
	}
	pub fn direct_children_of(&self, index: usize) -> HashSet<usize> {
		self.inner.iter().enumerate().skip(index + 1).filter_map(|(idx, node)| node.1.and_then(|i|
			match i == index {
				true => Some(idx),
				false => None
			}
		)).collect()
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn is_valid_idx(&self, index: usize) -> bool {
		return index == 0 || self[index].1.is_some()
	}

	// Inner exposures
	pub fn capacity(&self) -> usize {
		self.inner.capacity()
	}
	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.inner.iter().enumerate().filter(|(idx, _)| self.is_valid_idx(*idx)).map(|x| &x.1.0)
	}
	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.inner.shrink_to(min_capacity)
	}
	pub fn shrink_to_fit(&mut self) {
		self.inner.shrink_to_fit()
	}
	pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.inner.try_reserve(additional)
	}
	pub fn try_reserve_exact(&mut self, additional: usize ) -> Result<(), TryReserveError> {
		self.inner.try_reserve_exact(additional)
	}
}

impl<T> Index<usize> for Heap<T> {
	type Output = (T, Option<usize>);

	fn index(&self, index: usize) -> &Self::Output {
		&self.inner[index]
	}
}
impl<T> IndexMut<usize> for Heap<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.inner[index]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn make_test_heap() -> Heap<&'static str> {
		let mut heap = Heap::new("root");
		heap.insert("first child", 0);
		heap.insert("second child", 0);
		heap.insert("third child", 0);
		heap.insert("first grandchild", 1);
		heap.insert("second grandchild", 1);
		heap.insert("third grandchild", 2);
		heap.insert("fourth grandchild", 2);
		heap.insert("great grandchild", 4);
		heap
	}

	#[test]
	fn insert_test() {
		make_test_heap();
	}

	#[test]
	fn remove_test() {
		let mut heap = make_test_heap();
		heap.remove(1);
		assert_eq!(heap.len(), 5)
	}
	#[test]
	#[should_panic]
	fn remove_root_test() {
		let mut heap = make_test_heap();
		heap.remove(0);
	}

	#[test]
	fn get_children_test() {
		let mut children = HashSet::new();
		let mut heap = Heap::new("root");
		children.insert(heap.insert("first child", 0));
		children.insert(heap.insert("second child", 0));
		children.insert(heap.insert("third child", 0));
		heap.insert("first grandchild", 1);
		heap.insert("second grandchild", 1);
		heap.insert("third grandchild", 2);
		heap.insert("fourth grandchild", 2);
		heap.insert("great grandchild", 4);
		assert_eq!(heap.direct_children_of(0), children);
	}

	#[test]
	fn get_descendants_test() {
		let mut children = HashSet::new();
		let mut heap = Heap::new("root");
		heap.insert("first child", 0);
		heap.insert("second child", 0);
		heap.insert("third child", 0);
		children.insert(heap.insert("first grandchild", 1));
		children.insert(heap.insert("second grandchild", 1));
		heap.insert("third grandchild", 2);
		heap.insert("fourth grandchild", 2);
		children.insert(heap.insert("great grandchild", 4));
		assert_eq!(heap.descendants_of(1), children);
	}
}
