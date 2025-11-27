use core::{marker::PhantomData, ptr::NonNull};

pub struct Node<T>
{
  next: Link<T>,
  prev: Link<T>,
  elem: T,
}

pub type Link<T> = Option<NonNull<Node<T>>>;

pub struct List<T>
{
  front: Link<T>,
  back: Link<T>,
  len: usize,

  _phantom: PhantomData<T>,
}

pub struct CursorMut<'a, T>
{
  idx: Option<usize>,
  current: Link<T>,
  list: &'a mut List<T>,
}

impl<T> List<T>
{
  pub const fn new() -> Self
  {
    Self {
      front: None,
      back: None,
      len: 0,
      _phantom: PhantomData,
    }
  }

  // We take ptr to a node to avoid allocations, thats on the users side. (useful for allocators)
  pub fn push_front(&mut self, node: NonNull<Node<T>>)
  {
    unsafe {
      if let Some(old_front) = self.front
      {
        (*node.as_ptr()).next = Some(old_front); // Make sure the new node's back is now the old front
        (*old_front.as_ptr()).prev = Some(node); // and make sure the old front points back to the new one
      }
      else
      {
        self.back = Some(node); // If there is no front than the front and back will both point here
      }
      self.front = Some(node); // front is now the new node
      self.len += 1; // increase len
    }
  }

  pub fn push_back(&mut self, node: NonNull<Node<T>>)
  {
    unsafe {
      if let Some(old_back) = self.back
      {
        (*node.as_ptr()).prev = Some(old_back);
        (*old_back.as_ptr()).next = Some(node);
      }
      else
      {
        self.front = Some(node);
      }
      self.back = Some(node);
      self.len += 1;
    }
  }

  pub fn pop_front(&mut self) -> Link<T>
  {
    unsafe {
      self.front.map(|old_front| {
        self.front = (*old_front.as_ptr()).next; // set the front to the next node
        if let Some(new_front) = self.front
        {
          // make sure the new front isnt pointing to the old front
          (*new_front.as_ptr()).prev = None;
        }
        else
        {
          self.back = None
        }
        self.len -= 1;
        old_front
      })
    }
  }

  pub fn pop_back(&mut self) -> Link<T>
  {
    unsafe {
      self.back.map(|old_back| {
        self.back = (*old_back.as_ptr()).prev;
        if let Some(new_back) = self.back
        {
          (*new_back.as_ptr()).next = None;
        }
        else
        {
          self.front = None;
        }
        self.len -= 1;
        old_back
      })
    }
  }

  pub fn front_val(&self) -> Option<&T>
  {
    unsafe { self.front.map(|x| &(*x.as_ptr()).elem) }
  }
  pub fn front_val_mut(&mut self) -> Option<&mut T>
  {
    unsafe { self.front.map(|x| &mut (*x.as_ptr()).elem) }
  }

  pub fn back_val(&self) -> Option<&T>
  {
    unsafe { self.back.map(|x| &(*x.as_ptr()).elem) }
  }
  pub fn back_val_mut(&mut self) -> Option<&mut T>
  {
    unsafe { self.back.map(|x| &mut (*x.as_ptr()).elem) }
  }

  pub fn len(&self) -> usize
  {
    self.len
  }

  pub fn cursor_mut<'a>(&'a mut self) -> CursorMut<'a, T>
  {
    CursorMut::new(self)
  }

  pub fn empty(&self) -> bool
  {
    self.len == 0
  }
}

impl<T> Node<T>
{
  pub fn new(data: T) -> Self
  {
    Self {
      next: None,
      prev: None,
      elem: data,
    }
  }
  pub fn elem(&self) -> &T
  {
    &self.elem
  }
  pub fn elem_mut(&mut self) -> &mut T
  {
    &mut self.elem
  }
}

impl<'a, T> CursorMut<'a, T>
{
  pub(self) fn new(list: &'a mut List<T>) -> Self
  {
    Self {
      idx: None,
      current: None,
      list,
    }
  }

  pub fn move_next(&mut self)
  {
    if let Some(node) = self.current
    {
      self.idx = Some(self.idx.unwrap_or_default() + 1);
      self.current = unsafe { (*node.as_ptr()).next };
    }
    else
    {
      self.idx = Some(0);
      self.current = self.list.front;
    }
  }
  pub fn move_prev(&mut self)
  {
    if let Some(node) = self.current
    {
      self.idx = Some(self.idx.unwrap_or_default() - 1);
      self.current = unsafe { (*node.as_ptr()).prev };
    }
    else
    {
      self.idx = Some(self.list.len() - 1);
      self.current = self.list.back;
    }
  }

  pub fn current_value(&mut self) -> Option<&mut T>
  {
    self.current.map(|x| unsafe { &mut (*x.as_ptr()).elem })
  }

  pub fn remove(&mut self) -> Link<T>
  {
    let link_current = self.current;
    self.move_next();

    unsafe {
      if let Some(p_current) = link_current
      {
        self.idx = self.idx.map(|x| x - 1);

        let current = &mut (*p_current.as_ptr());
        let (o_prev, o_next) = (current.prev, current.next);

        match (o_prev, o_next)
        {
          (None, None) => self.list.pop_front(), // Only node in the list, therefore pop front will do the trick
          (None, Some(_)) => self.list.pop_front(), // its the front node
          (Some(_), None) => self.list.pop_back(), // its the back node
          (Some(p), Some(n)) =>
          {
            // Its the middle node, stitch the two nodes together
            (*p.as_ptr()).next = Some(n);
            (*n.as_ptr()).prev = Some(p);
            self.list.len -= 1;
            current.prev = None;
            current.next = None;
            link_current
          }
        }
      }
      else
      {
        link_current
      }
    }
  }

  pub fn insert_before(&mut self, node: NonNull<Node<T>>)
  {
    let link_current = self.current;
    unsafe {
      if let Some(p_current) = link_current
      {
        let current = &mut (*p_current.as_ptr());
        if let Some(p_prev) = current.prev
        {
          let prev = &mut (*p_prev.as_ptr());
          let new = &mut (*node.as_ptr());
          prev.next = Some(node);
          current.prev = Some(node);

          new.prev = Some(p_prev);
          new.next = Some(p_current);
          self.idx = self.idx.map(|x| x + 1);
          self.list.len += 1;
        }
        // at front
        else
        {
          self.list.push_front(node);
        }
      }
    }
  }
}

#[cfg(test)]
mod test
{
  use super::{List, Node};
  use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
  };
  use std::alloc::System;

  type NodeType = Node<usize>;
  const NODE_LAYOUT: Layout =
    unsafe { Layout::from_size_align_unchecked(size_of::<NodeType>(), align_of::<NodeType>()) };

  #[test]
  fn test_basic_front()
  {
    let mut list = List::new();

    // Try to break an empty list
    assert_eq!(list.len(), 0);
    assert_eq!(list.pop_front(), None);
    assert_eq!(list.len(), 0);

    unsafe {
      let ten = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
      let twe = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
      let thr = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
      let frt = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
      ten.write(Node::new(10));
      twe.write(Node::new(20));
      thr.write(Node::new(30));
      frt.write(Node::new(40));
      // Try to break a one item list
      list.push_front(ten);
      assert_eq!(list.len(), 1);
      assert_eq!(list.pop_front(), Some(ten));
      assert_eq!(list.len(), 0);
      assert_eq!(list.pop_front(), None);
      assert_eq!(list.len(), 0);

      // Mess around
      list.push_front(ten);
      assert_eq!(list.len(), 1);
      list.push_front(twe);
      assert_eq!(list.len(), 2);
      list.push_front(thr);
      assert_eq!(list.len(), 3);
      assert_eq!(list.pop_front(), Some(thr));
      assert_eq!(list.len(), 2);
      list.push_front(frt);
      assert_eq!(list.len(), 3);
      assert_eq!(list.pop_front(), Some(frt));
      assert_eq!(list.len(), 2);
      assert_eq!(list.pop_front(), Some(twe));
      assert_eq!(list.len(), 1);
      assert_eq!(list.pop_front(), Some(ten));
      assert_eq!(list.len(), 0);
      assert_eq!(list.pop_front(), None);
      assert_eq!(list.len(), 0);
      assert_eq!(list.pop_front(), None);
      assert_eq!(list.len(), 0);

      System.dealloc(ten.as_ptr() as *mut u8, NODE_LAYOUT);
      System.dealloc(twe.as_ptr() as *mut u8, NODE_LAYOUT);
      System.dealloc(thr.as_ptr() as *mut u8, NODE_LAYOUT);
      System.dealloc(frt.as_ptr() as *mut u8, NODE_LAYOUT);
    }
  }

  #[test]
  fn test_cursor_remove()
  {
    const COUNT: usize = 100;
    let mut alloc_vec = Vec::with_capacity(COUNT);
    let mut list = List::new();
    for i in 0..COUNT
    {
      let ptr = unsafe {
        let ret = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
        ret.as_ptr().write(Node::new(i));
        ret
      };
      list.push_front(ptr);
      alloc_vec.push(ptr);
    }

    let mut counter = 0;
    let start_idx = rand::random_range(0..list.len());
    let mut cursor = list.cursor_mut();
    cursor.move_next();
    for _ in 0..start_idx
    {
      cursor.move_next();
    }

    while cursor.current_value().is_some()
    {
      cursor.remove();
      counter += 1;
    }

    drop(cursor);
    while list.pop_front().is_some()
    {
      counter += 1;
    }
    assert!(counter == COUNT);

    while let Some(x) = alloc_vec.pop()
    {
      unsafe { System.dealloc(x.as_ptr() as *mut u8, NODE_LAYOUT) };
    }
  }

  #[test]
  fn test_cursor_insert_before()
  {
    const COUNT: usize = 100;
    const INSERT_MAX: usize = 20;
    let mut alloc_vec = Vec::with_capacity(COUNT);
    let mut list = List::new();
    for i in 0..COUNT
    {
      let ptr = unsafe {
        let ret = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
        ret.as_ptr().write(Node::new(i));
        ret
      };
      list.push_front(ptr);
      alloc_vec.push(ptr);
    }

    let start_idx = rand::random_range(0..list.len());
    let mut cursor = list.cursor_mut();
    cursor.move_next();
    for _ in 0..start_idx
    {
      cursor.move_next();
    }
    let insert_count = rand::random_range(1..INSERT_MAX);
    for i in 0..insert_count
    {
      let ptr = unsafe {
        let ret = NonNull::new(System.alloc(NODE_LAYOUT) as *mut NodeType).unwrap();
        ret.as_ptr().write(Node::new(i));
        ret
      };
      cursor.insert_before(ptr);
      alloc_vec.push(ptr);
    }
    drop(cursor);
    dbg!(list.len());
    dbg!(insert_count);
    assert!(list.len() == COUNT + insert_count);

    while let Some(x) = alloc_vec.pop()
    {
      unsafe { System.dealloc(x.as_ptr() as *mut u8, NODE_LAYOUT) };
    }
  }
}
