// Shader taken from https://github.com/johnbchron/neutron
// Credit belongs to the author, permission pending. 
// Should permission be denied, this shader will be removed.

fn cubic_bezier_sdf(
  P0: vec2<f32>,
  P1: vec2<f32>,
  P2: vec2<f32>,
  P3: vec2<f32>,
  p: vec2<f32>
) -> f32 {
  let A = -P0 + 3.0*P1 - 3.0*P2 + P3;
  let B = 3.0*(P0 - 2.0*P1 + P2);
  let C = 3.0*(P1 - P0);
  let D = P0;
    
  let a5 = 6.0*dot(A,A);
  let a4 = 10.0*dot(A,B);
  let a3 = 8.0*dot(A,C) + 4.0*dot(B,B);
  let a2 = 6.0*dot(A,D-p) + 6.0*dot(B,C);
  let a1 = 4.0*dot(B,D-p) + 2.0*dot(C,C);
  let a0 = 2.0*dot(C,D-p);
    
  // calculate distances to the control points
  // let d0 = length(p-P0);
  // let d1 = length(p-P1);
  // let d2 = length(p-P2);
  // let d3 = length(p-P3);
  // let d = min(d0, min(d1, min(d2,d3)));
    
    
  var t: f32 = 0.5;
       
  // iterate
  for (var i = 0; i < 10; i++) {
    let t2 = t*t;
    let t3 = t2*t;
    let t4 = t3*t;
    let t5 = t4*t;
    
    let f = a5*t5 + a4*t4 + a3*t3 + a2*t2 + a1*t + a0;
    let df = 5.0*a5*t4 + 4.0*a4*t3 + 3.0*a3*t2 + 2.0*a2*t + a1;
        
    t = t - f/df;
  }
    
  t = clamp(t, 0.0, 1.0);
    
  // get the point on the curve
  let P = A*t*t*t + B*t*t + C*t + D;
        
  // return min(length(p-P), min(d0, d3));
  return length(p-P);
}