/// Returns [0,1,..,N].
pub(crate) const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = i;
        i += 1;
    }
    arr
}

/// Returns the first element of a pair.
pub(crate) fn fst<A, B>(x: (A, B)) -> A {
    x.0
}

/// Returns the second element of a pair.
pub(crate) fn snd<A, B>(x: (A, B)) -> B {
    x.1
}

/// A testing macro which defines a wrapper struct for the given stark that
/// implements [`starky::stark::Stark`] using the given implementation with
/// one exception: `requires_ctls` returns false.
macro_rules! impl_stark_no_ctls {
    ($Stark:ty) => {
        ::paste::paste! {
            #[cfg(test)]
            #[derive(Clone, Copy, Default)]
            struct [<$Stark NoCtls>]<F, const D: usize>(pub $Stark<F, D>);

            #[cfg(test)]
            impl<
                F: ::plonky2::hash::hash_types::RichField
                + ::plonky2::field::extension::Extendable<D>,
                const D: usize
            > ::starky::stark::Stark<F, D> for [<$Stark NoCtls>]<F, D> {

                const COLUMNS: usize = <$Stark<F, D> as ::starky::stark::Stark<F, D>>::COLUMNS;
                const PUBLIC_INPUTS: usize = <$Stark<F, D> as ::starky::stark::Stark<F, D>>::PUBLIC_INPUTS;

                type EvaluationFrame<FE, P, const D2: usize>
                    = <$Stark<F, D> as ::starky::stark::Stark<F, D>>::EvaluationFrame<FE, P, D2>
                where
                    FE: ::plonky2::field::extension::FieldExtension<D2, BaseField = F>,
                    P: ::plonky2::field::packed::PackedField<Scalar = FE>;

                type EvaluationFrameTarget
                    = <$Stark<F, D> as ::starky::stark::Stark<F, D>>::EvaluationFrameTarget;

                fn eval_packed_generic<FE, P, const D2: usize>(
                    &self,
                    frame: &Self::EvaluationFrame<FE, P, D2>,
                    cc: &mut ::starky::constraint_consumer::ConstraintConsumer<P>,
                ) where
                    FE: ::plonky2::field::extension::FieldExtension<D2, BaseField = F>,
                    P: ::plonky2::field::packed::PackedField<Scalar = FE>,
                {
                    self.0.eval_packed_generic(frame, cc)
                }

                fn eval_packed_base<P: ::plonky2::field::packed::PackedField<Scalar = F>>(
                    &self,
                    frame: &Self::EvaluationFrame<F, P, 1>,
                    cc: &mut ::starky::constraint_consumer::ConstraintConsumer<P>,
                ) {
                    self.0.eval_packed_base(frame, cc)
                }

                fn eval_ext(
                    &self,
                    frame: &Self::EvaluationFrame<
                        <F as ::plonky2::field::extension::Extendable<D>>::Extension,
                        <F as ::plonky2::field::extension::Extendable<D>>::Extension,
                        D,
                    >,
                    cc: &mut ::starky::constraint_consumer::ConstraintConsumer<
                        <F as ::plonky2::field::extension::Extendable<D>>::Extension,
                    >,
                ) {
                    self.0.eval_ext(frame, cc)
                }


                fn eval_ext_circuit(
                    &self,
                    cb: &mut ::plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
                    frame: &Self::EvaluationFrameTarget,
                    cc: &mut ::starky::constraint_consumer::RecursiveConstraintConsumer<F, D>,
                ) {
                    self.0.eval_ext_circuit(cb, frame, cc);
                }

                fn constraint_degree(&self) -> usize {
                    self.0.constraint_degree()
                }

                fn quotient_degree_factor(&self) -> usize {
                    self.0.quotient_degree_factor()
                }

                fn num_quotient_polys(&self, cfg: &::starky::config::StarkConfig) -> usize {
                    self.0.num_quotient_polys(cfg)
                }

                fn fri_instance(
                    &self,
                    zeta: <F as ::plonky2::field::extension::Extendable<D>>::Extension,
                    g: F,
                    num_ctl_helpers: usize,
                    num_ctl_zs: Vec<usize>,
                    cfg: &::starky::config::StarkConfig,
                ) -> ::plonky2::fri::structure::FriInstanceInfo<F, D> {
                    self.0.fri_instance(zeta, g, num_ctl_helpers, num_ctl_zs, cfg)
                }

                fn fri_instance_target(
                    &self,
                    cb: &mut ::plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
                    zeta: ::plonky2::iop::ext_target::ExtensionTarget<D>,
                    g: F,
                    num_ctl_helper_polys: usize,
                    num_ctl_zs: usize,
                    cfg: &::starky::config::StarkConfig,
                ) -> ::plonky2::fri::structure::FriInstanceInfoTarget<D> {
                    self.0.fri_instance_target(cb, zeta, g, num_ctl_helper_polys, num_ctl_zs, cfg)
                }

                fn lookups(&self) -> Vec<::starky::lookup::Lookup<F>> {
                    self.0.lookups()
                }

                fn num_lookup_helper_columns(&self, cfg: &::starky::config::StarkConfig) -> usize {
                    self.0.num_lookup_helper_columns(cfg)
                }

                fn uses_lookups(&self) -> bool {
                    self.0.uses_lookups()
                }

                fn requires_ctls(&self) -> bool {
                    false
                }
            }

        }
    };
}
#[cfg(test)]
pub(crate) use impl_stark_no_ctls;
