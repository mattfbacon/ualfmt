#[derive(Debug)]
pub struct Interspersed<Around, Between> {
	rest: Vec<(Around, Between)>,
	last: Around,
}

pub enum AroundOrBetween<Around, Between> {
	Around(Around),
	Between(Between),
}

impl<Around, Between> Interspersed<Around, Between> {
	pub fn from_rest_and_last(rest: Vec<(Around, Between)>, last: Around) -> Self {
		Self { rest, last }
	}

	pub fn betweens_mut(&mut self) -> impl Iterator<Item = (Around, &mut Between, Around)> + '_
	where
		Around: Clone,
	{
		BetweensMut::new(self)
	}

	pub fn arounds_mut(&mut self) -> impl Iterator<Item = AroundsItem<'_, Around, Between>> + '_
	where
		Between: Clone,
	{
		AroundsMut::new(self)
	}

	pub fn iter_all(&self) -> impl Iterator<Item = AroundOrBetween<&Around, &Between>> + '_ {
		self
			.rest
			.iter()
			.flat_map(|(around, between)| {
				[
					AroundOrBetween::Around(around),
					AroundOrBetween::Between(between),
				]
			})
			.chain(std::iter::once(AroundOrBetween::Around(&self.last)))
	}
}

struct BetweensMut<'a, Around, Between> {
	inner: std::iter::Peekable<std::slice::IterMut<'a, (Around, Between)>>,
	last: &'a Around,
}

impl<'a, Around, Between> BetweensMut<'a, Around, Between> {
	fn new(container: &'a mut Interspersed<Around, Between>) -> Self {
		Self {
			inner: container.rest.iter_mut().peekable(),
			last: &container.last,
		}
	}
}

impl<'a, Around: Clone, Between> Iterator for BetweensMut<'a, Around, Between> {
	type Item = (Around, &'a mut Between, Around);

	fn next(&mut self) -> Option<Self::Item> {
		let (left_around, between): &'a mut (Around, Between) = self.inner.next()?;
		let right_around: Around = self
			.inner
			.peek()
			.map_or(self.last.clone(), |(around, _next_between)| around.clone());
		Some((left_around.clone(), between, right_around))
	}
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum AroundsItem<'a, Around, Between> {
	First(&'a mut Around, Between),
	Inner(Between, &'a mut Around, Between),
	Last(Between, &'a mut Around),
	/// will be emitted iff there is only a single `Around` in the `Interspersed`.
	Single(&'a mut Around),
}

impl<'a, Around, Between> AroundsItem<'a, Around, Between> {
	pub fn into_tuple(self) -> (Option<Between>, &'a mut Around, Option<Between>) {
		self.into()
	}
}

impl<'a, Around, Between> From<AroundsItem<'a, Around, Between>>
	for (Option<Between>, &'a mut Around, Option<Between>)
{
	fn from(item: AroundsItem<'a, Around, Between>) -> Self {
		match item {
			AroundsItem::First(around, after) => (None, around, Some(after)),
			AroundsItem::Inner(before, around, after) => (Some(before), around, Some(after)),
			AroundsItem::Last(before, around) => (Some(before), around, None),
			AroundsItem::Single(around) => (None, around, None),
		}
	}
}

struct AroundsMut<'a, Around, Between> {
	inner: std::slice::IterMut<'a, (Around, Between)>,
	last: Option<&'a mut Around>,
	prev_between: Option<Between>,
}

impl<'a, Around, Between> AroundsMut<'a, Around, Between> {
	fn new(container: &'a mut Interspersed<Around, Between>) -> Self {
		Self {
			inner: container.rest.iter_mut(),
			last: Some(&mut container.last),
			prev_between: None,
		}
	}
}

impl<'a, Around, Between: Clone> Iterator for AroundsMut<'a, Around, Between> {
	type Item = AroundsItem<'a, Around, Between>;

	fn next(&mut self) -> Option<Self::Item> {
		let item = if let Some((around, after_between)) = self.inner.next() {
			let after_between = after_between.clone();
			let prev_between = self.prev_between.replace(after_between.clone());
			if let Some(prev_between) = prev_between {
				AroundsItem::Inner(prev_between, around, after_between)
			} else {
				AroundsItem::First(around, after_between)
			}
		} else {
			let last = self.last.take()?;
			if let Some(prev_between) = self.prev_between.take() {
				AroundsItem::Last(prev_between, last)
			} else {
				AroundsItem::Single(last)
			}
		};
		Some(item)
	}
}

#[cfg(test)]
mod tests {
	use super::{AroundsItem, Interspersed};

	#[test]
	fn betweens() {
		let mut container = Interspersed::<i32, i32>::from_rest_and_last(vec![(1, 2), (3, 4)], 5);
		assert_eq!(
			container.betweens_mut().collect::<Vec<_>>(),
			[(1, &mut 2, 3), (3, &mut 4, 5)]
		);
	}

	#[test]
	fn betweens_single() {
		let mut container = Interspersed::<i32, i32>::from_rest_and_last(Vec::new(), 1);
		assert_eq!(container.betweens_mut().collect::<Vec<_>>(), []);
	}

	#[test]
	fn arounds() {
		let mut container = Interspersed::<i32, i32>::from_rest_and_last(vec![(1, 2), (3, 4)], 5);
		assert_eq!(
			container.arounds_mut().collect::<Vec<_>>(),
			[
				AroundsItem::First(&mut 1, 2),
				AroundsItem::Inner(2, &mut 3, 4),
				AroundsItem::Last(4, &mut 5)
			]
		);
	}

	#[test]
	fn arounds_single() {
		let mut container = Interspersed::<i32, i32>::from_rest_and_last(Vec::new(), 1);
		assert_eq!(
			container.arounds_mut().collect::<Vec<_>>(),
			[AroundsItem::Single(&mut 1)]
		);
	}
}
