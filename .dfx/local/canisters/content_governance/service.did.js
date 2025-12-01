export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'get_book_count' : IDL.Func([], [IDL.Nat64], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
