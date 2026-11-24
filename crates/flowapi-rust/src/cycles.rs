//! Which references close a cycle: Tarjan's strongly connected components, with an explicit
//! stack (G1). A struct that holds itself by value has no size, so such an edge is boxed; a
//! type alias that names itself is not allowed, so such an alias becomes a newtype.

/// The component of each node; two nodes on a common cycle share one.
pub(crate) fn components(nodes: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nodes];
    for &(a, b) in edges {
        if let Some(list) = adj.get_mut(a) {
            list.push(b);
        }
    }
    let unset = usize::MAX;
    let mut index = vec![unset; nodes];
    let mut low = vec![0; nodes];
    let mut on_stack = vec![false; nodes];
    let mut comp = vec![unset; nodes];
    let mut stack: Vec<usize> = Vec::new();
    let mut next = 0;
    let mut count = 0;
    for root in 0..nodes {
        if index.get(root).copied() != Some(unset) {
            continue;
        }
        // Each frame: a node and how many of its edges are walked.
        let mut frames: Vec<(usize, usize)> = vec![(root, 0)];
        set(&mut index, root, next);
        set(&mut low, root, next);
        next += 1;
        stack.push(root);
        set(&mut on_stack, root, true);
        while let Some(&mut (v, ref mut at)) = frames.last_mut() {
            let succ = adj.get(v).and_then(|l| l.get(*at)).copied();
            *at += 1;
            if let Some(w) = succ {
                if get(&index, w) == unset {
                    set(&mut index, w, next);
                    set(&mut low, w, next);
                    next += 1;
                    stack.push(w);
                    set(&mut on_stack, w, true);
                    frames.push((w, 0));
                } else if get(&on_stack, w) {
                    let m = get(&low, v).min(get(&index, w));
                    set(&mut low, v, m);
                }
                continue;
            }
            frames.pop();
            if let Some(&(parent, _)) = frames.last() {
                let m = get(&low, parent).min(get(&low, v));
                set(&mut low, parent, m);
            }
            if get(&low, v) == get(&index, v) {
                while let Some(w) = stack.pop() {
                    set(&mut on_stack, w, false);
                    set(&mut comp, w, count);
                    if w == v {
                        break;
                    }
                }
                count += 1;
            }
        }
    }
    comp
}

fn get<T: Copy + Default>(v: &[T], i: usize) -> T {
    v.get(i).copied().unwrap_or_default()
}

fn set<T>(v: &mut [T], i: usize, value: T) {
    if let Some(slot) = v.get_mut(i) {
        *slot = value;
    }
}

/// The edges that lie on a cycle: a self-loop, or both ends in one component.
pub(crate) fn on_cycle(nodes: usize, edges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let comp = components(nodes, edges);
    let mut out: Vec<(usize, usize)> = edges
        .iter()
        .copied()
        .filter(|&(a, b)| a == b || (comp.get(a).is_some() && comp.get(a) == comp.get(b)))
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}
