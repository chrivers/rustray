use super::geo_util::*;
use super::triangle::Triangle;

use obj::ObjData;

use bvh::bvh::BVH;

pub struct TriangleMesh<F: Float, M: Material<F=F>>
{
    tris: Vec<Triangle<F, M>>,
    bvh: BVH,
    aabb: AABB,
    ni: usize,
}

impl<F: Float, M: Material<F=F>> Bounded for TriangleMesh<F, M> {

    fn aabb(&self) -> AABB {
        self.aabb
    }

}

impl<F: Float, M: Material<F=F>> BHShape for TriangleMesh<F, M> {

    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }

}

impl<F: Float, M: Material<F=F> + Clone> Geometry<F> for TriangleMesh<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = bvh::ray::Ray::new(
            Point3::new(
                ray.pos.x.to_f32()?,
                ray.pos.y.to_f32()?,
                ray.pos.z.to_f32()?,
            ),
            Vector3::new(
                ray.dir.x.to_f32()?,
                ray.dir.y.to_f32()?,
                ray.dir.z.to_f32()?,
            )
        );
        let aabbs = self.bvh.traverse(&r, &self.tris);

        let mut dist = F::max_value();
        let mut hit: Option<Hit<F>> = None;
        for t in &aabbs {
            if let Some(curhit) = t.intersect(ray)
            {
                let curdist = ray.pos.distance(curhit.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }
        hit
    }
}

fn a2point<F: Float>(p: &[f32; 2]) -> Point<F> {
    point!(F::from_f32(p[0]),
           F::from_f32(p[1]))
}

fn a2vec<F: Float>(p: &[f32; 3]) -> Vector<F> {
    vec3!(F::from_f32(p[0]),
          F::from_f32(p[1]),
          F::from_f32(p[2]))
}

impl<F: Float, M: Material<F=F> + Clone> TriangleMesh<F, M>
{
    pub fn new(mut tris: Vec<Triangle<F, M>>) -> Self
    {
        debug!("building bvh for {} triangles..", tris.len());

        let mut aabb = AABB::empty();
        for tri in &tris {
            aabb.join_mut(&tri.aabb());
        }

        let bvh = BVH::build(&mut tris);

        TriangleMesh {
            tris,
            bvh,
            aabb,
            ni: 0,
        }
    }

    pub fn load_obj(obj: ObjData, pos: Vector<F>, scale: F, mat: M) -> Self
    {
        let mut corner = Vector::new(
            F::max_value(),
            F::max_value(),
            F::max_value()
        );

        for o in &obj.objects {
            for g in &o.groups {
                for poly in &g.polys {
                    for n in 0..(poly.0.len() - 1) {
                        corner.x = corner.x.min(F::from_f32(obj.position[poly.0[n].0][0]));
                        corner.y = corner.y.min(F::from_f32(obj.position[poly.0[n].0][1]));
                        corner.z = corner.z.min(F::from_f32(obj.position[poly.0[n].0][2]));
                    }
                }
            }
        }

        let mut tris: Vec<Triangle<F, M>> = vec![];
        for o in &obj.objects {
            for g in &o.groups {
                for poly in &g.polys {
                    for n in 1..(poly.0.len() - 1) {
                        /* FIXME: .unwrap() is a terrible when loading data from a file */
                        let tri = Triangle::new (
                            (a2vec(&obj.position[poly.0[0  ].0]) - corner) * scale + pos,
                            (a2vec(&obj.position[poly.0[n  ].0]) - corner) * scale + pos,
                            (a2vec(&obj.position[poly.0[n+1].0]) - corner) * scale + pos,

                            a2vec(&obj.normal[poly.0[0].2.unwrap()]).normalize(),
                            a2vec(&obj.normal[poly.0[n].2.unwrap()]).normalize(),
                            a2vec(&obj.normal[poly.0[n+1].2.unwrap()]).normalize(),

                            a2point(&obj.texture[poly.0[0].1.unwrap()]),
                            a2point(&obj.texture[poly.0[n].1.unwrap()]),
                            a2point(&obj.texture[poly.0[n+1].1.unwrap()]),

                            mat.clone(),
                        );
                        tris.push(tri);
                    }
                }
            }
        }

        info!("loaded .obj [index: {}, normal: {}, uv: {}, face: {}]", obj.position.len(), obj.normal.len(), obj.texture.len(), tris.len());

        TriangleMesh::new(tris)
    }

}
