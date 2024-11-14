use std::collections::VecDeque;

pub trait IndexOf<T> {
    fn index_of(&self, elem: &T) -> Option<usize>;
}
impl <T> IndexOf<T> for Vec<T> where 
    for<'a> &'a T: PartialEq<&'a T>
 { 
    fn index_of(&self, elem: &T) -> Option<usize> {
        return self.iter().position(|e| e==&elem);
        
    }
}