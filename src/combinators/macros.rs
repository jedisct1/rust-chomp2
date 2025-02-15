/// Macro to implement and run a parser iterator, it provides the ability to add
/// an extra state variable into it and also provide a size_hint as well as a
/// pre- and on-next hooks.
macro_rules! run_iter {
    (
        input:     $input:expr,
        parser:    $parser:expr,
        state:     $data_ty:ty : $data:expr,

        size_hint($size_hint_self:ident) $size_hint:block
        next($next_self:ident) {
            pre $pre_next:block
            on  $on_next:block
        }

        => $result:ident : $t:ty {
             $($pat:pat => $arm:expr),*$(,)*
        }
    ) => { {
        struct Iter<I: Input, T, E, F>
          where F: FnMut(I) -> ParseResult<I, T, E> {
            /// Last state of the parser
            state:  Option<E>,
            /// Parser to execute once for each iteration
            parser: F,
            /// Remaining buffer
            ///
            /// Wrapped in option to prevent two calls to destructors.
            buf:    Option<I>,
            /// Last good state.
            ///
            /// Wrapped in option to prevent two calls to destructors.
            mark:   I::Marker,
            /// Nested state
            data:   $data_ty,
            _t:     PhantomData<T>,
        }

        impl<I: Input, T, E, F> Iter<I, T, E, F>
          where F: FnMut(I) -> ParseResult<I, T, E> {
            #[inline]
            fn end_state(self) -> (I, $data_ty, I::Marker, Option<E>) {
                // TODO: Avoid branch, check if this can be guaranteed to always be Some(T)
                (self.buf.expect("Iter.buf was None"), self.data, self.mark, self.state)
            }
        }

        impl<I: Input, T, E, F> Iterator for Iter<I, T, E, F>
          where F: FnMut(I) -> ParseResult<I, T, E> {
            type Item = T;

            #[inline]
            fn size_hint(&$size_hint_self) -> (usize, Option<usize>) {
                $size_hint
            }

            #[inline]
            fn next(&mut $next_self) -> Option<Self::Item> {
                $pre_next

                // TODO: Remove the branches here (ie. take + unwrap)
                let i = $next_self.buf.take().expect("Iter.buf was None");

                // TODO: Any way to prevent marking here since it is not used at all times?
                $next_self.mark = i.mark();

                match ($next_self.parser)(i).into_inner() {
                    (b, Ok(v)) => {
                        $next_self.buf = Some(b);

                        $on_next

                        Some(v)
                    },
                    (b, Err(e)) => {
                        $next_self.buf   = Some(b);
                        $next_self.state = Some(e);

                        None
                    },
                }
            }
        }

        // TODO: Not always used
        let mark = $input.mark();

        let mut iter = Iter {
            state:  None,
            parser: $parser,
            buf:    Some($input),
            mark,
            data:   $data,
            _t:     PhantomData,
        };

        let $result: $t = FromIterator::from_iter(iter.by_ref());

        match iter.end_state() {
            $($pat => $arm),*
        }
    } }
}

/// Version of run_iter which allows for an additional parser to be run which
/// can terminate iteration early.
macro_rules! run_iter_till {
    (
        input:     $input:expr,
        parser:    $parser:expr,
        end:       $end:expr,
        state:     $data_ty:ty : $data:expr,

        size_hint($size_hint_self:ident) $size_hint:block
        next($next_self:ident) {
            pre $pre_next:block
            on  $on_next:block
        }

        => $result:ident : $t:ty {
             $($pat:pat => $arm:expr),*$(,)*
        }
    ) => { {
        enum EndStateTill<E> {
            Error(E),
            Incomplete,
            EndSuccess,
        }

        /// Iterator used by `many_till` and `many1`.
        struct IterTill<I: Input, T, U, E, F, P, N>
          where E: From<N>,
                P: FnMut(I) -> ParseResult<I, T, E>,
                F: FnMut(I) -> ParseResult<I, U, N> {
            state:  EndStateTill<E>,
            parser: P,
            end:    F,
            buf:    Option<I>,
            data:   $data_ty,
            _t:     PhantomData<(T, U, N)>,
        }

        impl<I: Input, T, U, E, F, P, N> IterTill<I, T, U, E, F, P, N>
          where E: From<N>,
                P: FnMut(I) -> ParseResult<I, T, E>,
                F: FnMut(I) -> ParseResult<I, U, N> {
            /// Destructures the iterator returning the position just after the last successful parse as
            /// well as the state of the last attempt to parse data.
            #[inline]
            fn end_state(self) -> (I, $data_ty, EndStateTill<E>) {
                // TODO: Avoid branch, check if this can be guaranteed to always be Some(T)
                (self.buf.expect("Iter.buf was None"), self.data, self.state)
            }
        }

        impl<I: Input, T, U, E, F, P, N> Iterator for IterTill<I, T, U, E, F, P, N>
          where E: From<N>,
                P: FnMut(I) -> ParseResult<I, T, E>,
                F: FnMut(I) -> ParseResult<I, U, N> {
            type Item = T;

            #[inline]
            fn size_hint(&$size_hint_self) -> (usize, Option<usize>) {
                $size_hint
            }

            #[inline]
            fn next(&mut $next_self) -> Option<Self::Item> {
                $pre_next

                // TODO: Remove the branches here (ie. take + unwrap)
                let i = $next_self.buf.take().expect("Iter.buf was None");

                match ($next_self.parser)(i).into_inner() {
                    (b, Ok(v)) => {
                        $next_self.buf = Some(b);

                        $on_next

                        Some(v)
                    },
                    (b, Err(e)) => {
                        $next_self.buf   = Some(b);
                        $next_self.state = EndStateTill::Error(e);

                        None
                    },
                }
            }
        }

        let mut iter = IterTill {
            state:  EndStateTill::Incomplete,
            parser: $parser,
            end:    $end,
            buf:    Some($input),
            data:   $data,
            _t:     PhantomData,
        };

        let $result: $t = FromIterator::from_iter(iter.by_ref());

        match iter.end_state() {
            $($pat => $arm),*
        }
    } }
}

/// Used with `run_iter_till!` macro to attempt to end iteration early. If the
/// test succeeds the buffer position will be updated and the state set to
/// `EndStateTill::EndSuccess` and a `None` will be returned, stopping the
/// iteration. If the test fails execution continues.
macro_rules! iter_till_end_test {
    ($the_self:ident) => {{
        // TODO: Remove the branches here (ie. take + unwrap)
        let i = $the_self.buf.take().expect("Iter.buf was None");
        let m = i.mark();

        match ($the_self.end)(i).into_inner() {
            (b, Ok(_)) => {
                $the_self.buf = Some(b);
                $the_self.state = EndStateTill::EndSuccess;

                return None;
            }
            // Failed to end, restore and continue
            (b, Err(_)) => $the_self.buf = Some(b.restore(m)),
        }
    }};
}
