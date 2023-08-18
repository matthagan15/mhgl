Just some notes from working with mhgl for the error correcting code stuff:
- Never have the user interact with a `HgBasis` object. This is a pain in the ass.
- For the simplest HGraph it doesn't make sense to have multiple edges, even though it is possible.
- Also just use Uuid crate instead of going back and forth from u128. 
- for HGraph, also need to be able to query if an edge exists from just a slice or vec of uuids.
- implement indexing for the basis of the type. For example, I should be able to do hg[vec![a,b,c]] and have it return
true or false based on the edge being present or not.
