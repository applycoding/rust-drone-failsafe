use failsafe_core::{ConvexPolygon, Point2};

#[test]
fn square_contains_center() {
    let pts = [
        Point2 {
            n: -100.0,
            e: -100.0,
        },
        Point2 {
            n: -100.0,
            e: 100.0,
        },
        Point2 { n: 100.0, e: 100.0 },
        Point2 {
            n: 100.0,
            e: -100.0,
        },
    ];
    let poly = ConvexPolygon::new(&pts).unwrap();
    let c = Point2 { n: 0.0, e: 0.0 };
    assert!(poly.contains(c));
    assert!(poly.signed_distance(c) >= 0.0);
}

#[test]
fn boundary_counts_inside() {
    let pts = [
        Point2 { n: 0.0, e: 0.0 },
        Point2 { n: 100.0, e: 0.0 },
        Point2 { n: 50.0, e: 100.0 },
    ];
    let poly = ConvexPolygon::new(&pts).unwrap();
    let on_edge = Point2 { n: 50.0, e: 0.0 };
    assert!(poly.contains(on_edge));
}
