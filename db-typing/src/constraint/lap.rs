/// the ability to enumerate all sets in a class of non-overlapping sets
pub trait LapLock {
    type Set;
    type Tok;
    type Err;

    /// freeze all overlapping sets, get a token for this lock, Ok(None) means blocked
    fn lock(&self, set: &Self::Set) -> Result<Option<(Vec<&Self::Set>, Self::Tok)>, Self::Err>;

    /// free the frozen sets with a given token
    fn free(&self, tok: Self::Tok) -> Result<(), Self::Err>;

    /// insert a set into this structure, the set must be the one who generated the lock token
    fn insert(&self, set: Self::Set, tok: &Self::Tok) -> Result<(), Self::Err>;

    /// remove a set from this structure, the set should be one of the sets returned along with the token
    fn remove(&self, set: &Self::Set, tok: &Self::Tok) -> Result<Self::Set, Self::Err>;
}