# The Architecture

At a high level, there are three broad Apicius phases:

```
+-------+   +-------+   +--------+
| parse |-->| check |-->| render |
+-------+   +-------+   +--------+
```

If you want to get _real_ pedantic here, the `check` phase is also split into two sub-phases:

```
 +-------------------------------+
 |            check              |
 |  +---------+     +---------+  |
--->| analyze |---->| reverse |---->
 |  +---------+     +---------+  |
 +-------------------------------+
```

Each of these has a simple goal:

- The `parse` phase goes from a raw string to a raw representation of what appears in the program, essentially a set of paths that represent sequences of steps.
- The `check` phase goes from that raw representation to a tree representation, which starts at the end of the recipe and represents subsequent steps as child nodes.
    - The `analyze` phase in particular creates a representation which is amenable to finding invariant problems: we want our charts to be trees, since 1. that's a reasonable assumption for _most_ recipes and 2. that makes it a lot easier to graph. (Examples where we _might_ want to represent non-trees are when a given sub-preparation is used in multiple places—for example, boiling pasta, then using the pasta one place and the water another—and in simple repeated directions, but I think it's fair to not support that yet.)
    - The `reverse` phase relies on the invariants being true but will then produce the final tree representation, which we call a `BackwardTree` since it starts at the last step and traversing it brings you closer to the beginning of the recipe.
- Finally, we have a (WIP) set of renderers which take the `BackwardTree` and turn it into pretty formats.

# The `State` Type

Much of the `State` type is overkill. This project could have used a _much_ simpler set of data structures, but I wanted to use `apicius` as a testing ground for applying techniques I've used for high-performance compilers in [Sorbet]() before deploying them in a more complicated compiler setting.

So one thing that it does is it creates packed representations of most data and passes around `Ref` types which act as indices into those tables. In a compiler setting, this is a net win because it means we rarely do string comparisons except for during the initial interning: we theoretically benefit from that a _little_ here during the analysis phases, but we rarely deal with recipes large enough to really care.

This also makes printing a lot harder. In particular, a given structure no longer has enough data to really print what it means. Consequently, we rely on the `Printable` struct, which bundles a reference to a value with a reference to the `State` type, to create a thing which can be `Debug`'d. To make this easier, stuff which can be printed in a `Printable` struct (which requires an explicit implementation of `fmt::Debug` for `Printable<T>`) also usually derives the `ToPrintable` trait, which gives it the `printable` method. If you want to print a representation of something with the actual strings (as opposed to indices into the string interned table), you can usually use something like `println!("{:?}", my_value.prinable(&state))` to do so.

Anyway, if I were practical or reasonable, I'd probably undo some of the unnecessary interning and indirection, but I'm keeping it and instead might experiment with ways to make it more practical. Or not. Who knows.

# The `BackwardTree` Type

Before I get into the specifics of the type, let's back up and think about some concerns we might have when graphing a recipe.

Imagine we've got a simple recipe for sautéed mushrooms where we want to be sure we add the garlic to the oil _before_ we add the mushrooms:

```apicius
sauteed mushrooms {
  garlic    -> mince -> $oil -> fry -> $add -> sautee -> <>;
  olive oil          -> $oil;
  mushrooms -> chop                 -> $add;
}
```

We want to eventually turn this into different visual representations. For example, the table representation might look like this:

```
+-----------+-------+------+--------+--+
| garlic    | mince |      |        |  |
+-------------------+ fry  |        |  |
| olive oil         |      | sautee |<>|
+-------------------+------+        |  |
| mushrooms         | chop |        |  |
+-------------------+------+--------+--+
```

How do we put together this chart? In particular, how do we track the _spacing_ of things? Let's assume we're using HTML for this (which is how the table renderer is implemented): we need to know the `rowspan` and `colspan` of each cell, since a `colspan` of `n` means "make this cell extend the next `n-1` cells to the right", while a `rowspan` is "make this cell extend over the next `n-1` cells below".

So let's rewrite the above chart _without_ extending the cells, but with `(colspan, rowspan)` pairs indicating how big each cell should be, and _entirely removing_ the cells that shouldn't exist:

```
+-----+-----+-----+-----+-----+
| 1,1 | 1,1 | 1,2 | 1,3 | 1,3 |
+-----+-----+-----+-----+-----+
| 2,1 |
+-----+-----+
| 2,1 | 1,1 |
+-----+-----+
```

(You can by hand convert this into the appropriate `table`/`tr`/`td` cells and see that it prodouces the table we want!)

Okay, great! How do we _get_ those numbers, though?

Luckily, these numbers are all related to the shape of the tree. The rowspan (i.e. vertical stretch) is probably the easiest here. How tall is a cell going to be? It depends on how many _distinct ingredients_ are going into that node. In the case of the `mince` step, the only input is garlic, so it'll be 1. The `fry` step will be 2, because it takes the garlic _and_ oil, and the `sautée` step will be 3, because it takes the garlic _and_ oil _and_ mushrooms.

Keep in mind that this is _different_ from _how many inputs it has_, because it's transitive. The `sautée` step only takes in two inputs—the result of frying and the mushrooms—but it needs needs to have the height of 3.

The colspan (i.e. the horizontal stretch) is a bit trickier to visualize. Let's look only at the inputs to `fry` for a moment:

```
+-----------+-------+------+
| garlic    | mince |      |
+-------------------+ fry  |
| olive oil         |      |
+-------------------+------+
```

We can clearly see that `olive oil` _needs_ to stretch out for two cells. We're not mincing the olive oil—this recipe isn't _that_ modernist—and we don't really have anywhere else for it to go and keep the table shape, so it's pretty much forced to have a width of 2 here. In this case, it's because we have a parallel branch—another input to `fry`—that has 2. So the width of a cell is based on the _maximum width of adjacent branches_.

But it's also non-local: in particular, it depends on other branches. If we feed `fry` into a different node that itself has, say, _five_ input steps, then we'll have to extend `olive oil` even further to compensate.

```
+---+---+---+-------+------+-----+
| A | B | C |   D   |  E   |     |
+-----------+-------+------+     |
| garlic    | mince |      | new |
+-------------------+ fry  |     |
| olive oil         |      |     |
+-------------------+------+-----+
```

Now, `olive oil` needs a width of 4 (and `garlic` here is getting a width of 3: in the current table format, we always extend ingredients to the right rather than making cooking steps extend because it's easier.)

So that means we need to know something about the _maximum depth_ of each branch in order to compute this. That way, with `new`, we know that we'll need to extend every single sub-branch so that it spans 5 cells. (Or more, if that feeds into another node with a greater maximum depth!) This won't be sufficient to render in isolation, but this information can be passed down recursively to let child nodes know how much space they need to fill up.

So given all that, we can look at the `BackwardTree` and figure out why it has the data it does! The `BackwardTree` looks like this:

```rust
pub struct BackwardTree {
    // the steps associated with this node
    pub actions: Vec<ActionStep>,
    // the ingredients that feed into this node
    pub ingredients: Vec<IngredientRef>,

    // the numbers described above
    pub size: usize,
    pub max_depth: usize,

    // the children of this tree
    pub paths: Vec<BackwardTree>,
}
```

The `size` here is "how many input ingredients feed into this". We will always maintain the invariant that:

```rust
node.size == node.paths.map(|n| n.size).sum()
```

Similarly, `max_depth` is the longest chain of actions (_not_ including the ingredient cells) of _any_ child _plus_ the number of actions in that node. We will always maintain the invariant that

```rust
node.max_depth == node.actions.len() + node.paths.map(|n| n.max_depth).max()
```

Another invariant we maintain is that `node.ingredients` will be empty _iff_ `node.paths` is non-empty: that is to say, we either have child nodes _or_ we have ingredients, but never both. If there are ingredients, we've reached the end of a tree. Additionally, if both `actions` and `ingredients` are empty, we're looking at the root (or the finished recipe.)

So, given all that, what does our recipe above look like as a `BackwardTree`? Let's repeat the recipe here, for ease of reading:

```apicius
sauteed mushrooms {
  garlic    -> mince -> $oil -> fry -> $add -> sautee -> <>;
  olive oil          -> $oil;
  mushrooms -> chop                 -> $add;
}
```

And the `BackwardTree` will look something like this, omitting `actions` and `ingredients` if they're empty and using a visual tree to represent `paths`:

```
size: 3, max_depth: 3
 |
 +-size: 3, max_depth: 3, actions: [sautee]
    |
    +-size: 2, max_depth: 2, actions: [fry]
    |  |
    |  +-size: 1, max_depth: 1, actions: [mince], ingredients: [garlic]
    |  |
    |  +-size: 1, max_depth: 0, ingredients: [oil]
    |
    +--size: 1, max_depth: 1, actions: [chop], ingredients: [mushrooms]
```
