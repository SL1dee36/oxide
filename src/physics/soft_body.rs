use crate::assets::mesh::Mesh3D;
use crate::renderer::buffer::Vertex3D;
use glam::Vec3;

/// Single point mass particle simulating Position-Based Dynamics (PBD).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoftParticle {
    /// Current world position ($\mathbf{x}$).
    pub position: Vec3,
    /// Previous world position used for velocity estimation.
    pub old_position: Vec3,
    /// Predicted position after external forces integration ($\mathbf{x}^*$).
    pub predicted_position: Vec3,
    /// Linear velocity ($\mathbf{v}$).
    pub velocity: Vec3,
    /// Mass in kilograms.
    pub mass: f32,
    /// Inverse mass ($w = 1 / m$), 0.0 for fixed/pinned particles.
    pub inv_mass: f32,
}

impl SoftParticle {
    /// Creates a dynamic particle with specified initial position and mass.
    pub fn new(position: Vec3, mass: f32) -> Self {
        let safe_mass = mass.max(1e-5);
        Self {
            position,
            old_position: position,
            predicted_position: position,
            velocity: Vec3::ZERO,
            mass: safe_mass,
            inv_mass: 1.0 / safe_mass,
        }
    }

    /// Creates an immobile pinned particle unaffected by external forces or constraints.
    pub fn new_fixed(position: Vec3) -> Self {
        Self {
            position,
            old_position: position,
            predicted_position: position,
            velocity: Vec3::ZERO,
            mass: 0.0,
            inv_mass: 0.0,
        }
    }

    /// Returns `true` if this particle is immovable ($w = 0$).
    #[inline]
    pub fn is_fixed(&self) -> bool {
        self.inv_mass == 0.0
    }
}

/// Bilateral distance constraint between two particles in a Position-Based Dynamics lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistanceConstraint {
    /// Index of first particle in `SoftBody::particles`.
    pub particle_a: usize,
    /// Index of second particle in `SoftBody::particles`.
    pub particle_b: usize,
    /// Rest length at zero deformation ($d_0$).
    pub rest_length: f32,
    /// Spring stiffness parameter in [0.0, 1.0].
    pub stiffness: f32,
}

impl DistanceConstraint {
    /// Constructs a distance constraint between particles `a` and `b`.
    pub fn new(particle_a: usize, particle_b: usize, rest_length: f32, stiffness: f32) -> Self {
        Self {
            particle_a,
            particle_b,
            rest_length,
            stiffness: stiffness.clamp(0.0, 1.0),
        }
    }
}

/// Backwards-compatibility alias for [`DistanceConstraint`].
pub type SpringConstraint = DistanceConstraint;

/// Deformable soft body dynamics component based on Position-Based Dynamics (PBD) and XPBD.
#[derive(Debug, Clone)]
pub struct SoftBody {
    /// Particle point masses forming the deformable lattice.
    pub particles: Vec<SoftParticle>,
    /// Structural, shear, and bend distance constraints.
    pub constraints: Vec<DistanceConstraint>,
    /// Number of sub-steps per frame for PBD numerical stability (default: 4 to 8).
    pub substeps: u32,
    /// XPBD compliance parameter alpha (0.0 = completely rigid constraints, > 0.0 = compliant/elastic).
    pub compliance: f32,
    /// Volumetric internal pressure coefficient for closed meshes (e.g. inflatable balls).
    pub pressure: f32,
    /// Velocity damping ratio applied during particle integration.
    pub damping: f32,
}

impl SoftBody {
    /// Creates an empty soft body with default numerical parameters.
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            constraints: Vec::new(),
            substeps: 5,
            compliance: 0.0,
            pressure: 0.0,
            damping: 0.998,
        }
    }

    /// Configures the number of simulation sub-steps per physics update.
    pub fn with_substeps(mut self, substeps: u32) -> Self {
        self.substeps = substeps.max(1);
        self
    }

    /// Configures the XPBD compliance parameter $\alpha$ (elastic softness).
    pub fn with_compliance(mut self, compliance: f32) -> Self {
        self.compliance = compliance.max(0.0);
        self
    }

    /// Configures the velocity damping ratio.
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping.clamp(0.0, 1.0);
        self
    }

    /// Appends a new distance constraint between two particles.
    pub fn add_constraint(&mut self, constraint: DistanceConstraint) {
        self.constraints.push(constraint);
    }

    /// Backward-compatibility accessor for distance constraints.
    #[inline]
    pub fn springs(&self) -> &[DistanceConstraint] {
        &self.constraints
    }

    /// Constructs a soft body initialized from unique vertex positions of a [`Mesh3D`].
    pub fn from_mesh(mesh: &Mesh3D, particle_mass: f32, stiffness: f32) -> Self {
        let mut particles = Vec::with_capacity(mesh.vertices.len());
        for v in &mesh.vertices {
            particles.push(SoftParticle::new(Vec3::from_array(v.position), particle_mass));
        }

        let mut constraints = Vec::new();
        // Add constraints along all triangle edges
        for chunk in mesh.indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
                if a < particles.len() && b < particles.len() {
                    let d = (particles[a].position - particles[b].position).length();
                    constraints.push(DistanceConstraint::new(a, b, d, stiffness));
                }
            }
        }

        Self {
            particles,
            constraints,
            substeps: 5,
            compliance: 0.0,
            pressure: 0.0,
            damping: 0.998,
        }
    }

    /// Generates a planar 2D/3D cloth sheet mass-spring grid with a corresponding [`Mesh3D`].
    pub fn cloth(
        width: f32,
        height: f32,
        res_x: usize,
        res_y: usize,
        mass: f32,
        stiffness: f32,
    ) -> (Self, Mesh3D) {
        let res_x = res_x.max(2);
        let res_y = res_y.max(2);
        let particle_mass = mass / (res_x * res_y) as f32;

        let mut particles = Vec::with_capacity(res_x * res_y);
        let mut vertices = Vec::with_capacity(res_x * res_y);
        let mut indices = Vec::with_capacity((res_x - 1) * (res_y - 1) * 6);
        let mut constraints = Vec::new();

        let dx = width / (res_x - 1) as f32;
        let dy = height / (res_y - 1) as f32;
        let start_x = -width * 0.5;
        let start_y = height * 0.5;

        for y in 0..res_y {
            for x in 0..res_x {
                let pos = Vec3::new(start_x + x as f32 * dx, start_y - y as f32 * dy, 0.0);
                // Pin top corners by default
                let is_pinned = y == 0 && (x == 0 || x == res_x - 1);
                let p = if is_pinned {
                    SoftParticle::new_fixed(pos)
                } else {
                    SoftParticle::new(pos, particle_mass)
                };
                particles.push(p);

                let u = x as f32 / (res_x - 1) as f32;
                let v = y as f32 / (res_y - 1) as f32;
                vertices.push(Vertex3D::new(pos.to_array(), [0.0, 0.0, 1.0], [u, v]));
            }
        }

        let idx = |x: usize, y: usize| -> usize { y * res_x + x };

        for y in 0..res_y {
            for x in 0..res_x {
                let current = idx(x, y);

                // Structural horizontal constraint
                if x + 1 < res_x {
                    let right = idx(x + 1, y);
                    let len = (particles[current].position - particles[right].position).length();
                    constraints.push(DistanceConstraint::new(current, right, len, stiffness));
                }
                // Structural vertical constraint
                if y + 1 < res_y {
                    let down = idx(x, y + 1);
                    let len = (particles[current].position - particles[down].position).length();
                    constraints.push(DistanceConstraint::new(current, down, len, stiffness));
                }
                // Shear diagonal constraints
                if x + 1 < res_x && y + 1 < res_y {
                    let diag1 = idx(x + 1, y + 1);
                    let len1 = (particles[current].position - particles[diag1].position).length();
                    constraints.push(DistanceConstraint::new(current, diag1, len1, stiffness * 0.8));

                    let diag2_a = idx(x + 1, y);
                    let diag2_b = idx(x, y + 1);
                    let len2 =
                        (particles[diag2_a].position - particles[diag2_b].position).length();
                    constraints.push(DistanceConstraint::new(diag2_a, diag2_b, len2, stiffness * 0.8));
                }
                // Bending constraints (two steps apart)
                if x + 2 < res_x {
                    let right2 = idx(x + 2, y);
                    let len = (particles[current].position - particles[right2].position).length();
                    constraints.push(DistanceConstraint::new(current, right2, len, stiffness * 0.5));
                }
                if y + 2 < res_y {
                    let down2 = idx(x, y + 2);
                    let len = (particles[current].position - particles[down2].position).length();
                    constraints.push(DistanceConstraint::new(current, down2, len, stiffness * 0.5));
                }

                // Quad triangles
                if x + 1 < res_x && y + 1 < res_y {
                    let i0 = current as u32;
                    let i1 = idx(x + 1, y) as u32;
                    let i2 = idx(x + 1, y + 1) as u32;
                    let i3 = idx(x, y + 1) as u32;

                    indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
                }
            }
        }

        let soft_body = Self {
            particles,
            constraints,
            substeps: 5,
            compliance: 0.0,
            pressure: 0.0,
            damping: 0.998,
        };
        let mesh = Mesh3D::new(vertices, indices);

        (soft_body, mesh)
    }

    /// Sets or removes pinned state of particle at `index`.
    pub fn set_particle_fixed(&mut self, index: usize, fixed: bool) {
        if let Some(p) = self.particles.get_mut(index) {
            if fixed {
                p.inv_mass = 0.0;
                p.velocity = Vec3::ZERO;
            } else if p.mass > 0.0 {
                p.inv_mass = 1.0 / p.mass;
            } else {
                p.mass = 0.1;
                p.inv_mass = 10.0;
            }
        }
    }

    /// Applies a continuous external force to all dynamic particles in this soft body.
    pub fn apply_force(&mut self, force: Vec3) {
        for p in &mut self.particles {
            if p.inv_mass > 0.0 {
                p.velocity += force * p.inv_mass;
            }
        }
    }

    /// Synchronizes updated particle coordinates back into host [`Mesh3D`] vertex buffers and recomputes normals.
    pub fn sync_to_mesh(&self, mesh: &mut Mesh3D) {
        let count = mesh.vertices.len().min(self.particles.len());
        for i in 0..count {
            mesh.vertices[i].position = self.particles[i].position.to_array();
        }

        // Recalculate vertex normals from face cross products
        for v in &mut mesh.vertices {
            v.normal = [0.0, 0.0, 0.0];
        }

        for chunk in mesh.indices.as_chunks::<3>().0 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 < count && i1 < count && i2 < count {
                let p0 = self.particles[i0].position;
                let p1 = self.particles[i1].position;
                let p2 = self.particles[i2].position;

                let normal = (p1 - p0).cross(p2 - p0);
                if normal.length_squared() > 1e-8 {
                    let n = normal.normalize();
                    for idx in [i0, i1, i2] {
                        mesh.vertices[idx].normal[0] += n.x;
                        mesh.vertices[idx].normal[1] += n.y;
                        mesh.vertices[idx].normal[2] += n.z;
                    }
                }
            }
        }

        for v in &mut mesh.vertices {
            let n = Vec3::from_array(v.normal);
            if n.length_squared() > 1e-8 {
                v.normal = n.normalize().to_array();
            } else {
                v.normal = [0.0, 1.0, 0.0];
            }
        }
    }
}

impl Default for SoftBody {
    fn default() -> Self {
        Self::new()
    }
}
