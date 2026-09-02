use std::ops::{Add, Div, Mul, Sub};

use num_traits::AsPrimitive;

// pub fn linspace<F>(start: F, end: F, count: usize) -> impl Iterator<Item = F>
//     where
//     F: Copy + 'static +Add<Output = F> + Sub<Output = F> + Div<Output = F> + Mul<Output=F>,
//     usize: AsPrimitive<F>,
// {
//     (0..count).map(move |f| start + ((end-start) * f.as_() / (count-1).as_() ))
// }

//TODO: its annoying that we still have to state what S is with vec's e.g.
// = note: multiple `impl`s satisfying `Vec4: std::ops::Mul<_>` found in the `glam` crate:
//         - impl std::ops::Mul for Vec4;
//         - impl std::ops::Mul<&Vec4> for Vec4;
//         - impl std::ops::Mul<&f32> for Vec4;
//         - impl std::ops::Mul<f32> for Vec4;
// maybe just write a veclinspace
pub fn linspace<S, V>(start: V, end: V, count: usize) -> impl Iterator<Item = V>
where
    S: Copy + 'static + Div<Output = S>,
    V: Copy
        + 'static
        + Add<Output = V>
        + Sub<Output = V>
        + Mul<Output = V>
        + Div<Output = V>
        + Mul<S, Output = V>,
    usize: AsPrimitive<S>,
{
    //chain start and end values to avoid FP errors

    //TODO: microbenchmark - see https://nnethercote.github.io/perf-book/benchmarking.html
    //may be faster with if statement rather than chaining
    //see criterion crate for benchmarking

    std::iter::once(start)
        .chain((1..count - 1).map(move |f| start + ((end - start) * (f.as_() / (count - 1).as_()))))
        .chain(std::iter::once(end))
}

#[test]
fn test_linspace() {
    let v: Vec<_> = linspace::<f32, f32>(0.0, 1.0, 5).collect();
    println!("{:?}", v);
}
