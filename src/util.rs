/// Returns [0,1,..,N]
pub(crate) const fn indices_arr<const N: usize>() -> [usize; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = i;
        i += 1;
    }
    arr
}

pub(crate) fn fst<A, B>(x: (A, B)) -> A {
    x.0
}

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
            struct [<$Stark NoCtls>]<F, const D: usize>($Stark<F, D>);

            #[cfg(test)]
            impl<
                F: ::plonky2::hash::hash_types::RichField
                + ::plonky2::field::extension::Extendable<D>,
                const D: usize
            > ::starky::stark::Stark<F, D> for [<$Stark NoCtls>]<F, D> {
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

                fn lookups(&self) -> Vec<::starky::lookup::Lookup<F>> {
                    self.0.lookups()
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
