export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'parent_shard_id' : IDL.Principal });
  const ArchivedTransaction = IDL.Record({
    'transaction_type' : IDL.Text,
    'metadata' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Nat64,
    'sequence' : IDL.Nat64,
    'archived_at' : IDL.Nat64,
  });
  const ArchiveStats = IDL.Record({
    'is_full' : IDL.Bool,
    'size_bytes' : IDL.Nat64,
    'parent_shard' : IDL.Principal,
    'entry_count' : IDL.Nat64,
    'next_archive' : IDL.Opt(IDL.Principal),
  });
  const TransactionToArchive = IDL.Record({
    'transaction_type' : IDL.Text,
    'metadata' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'amount' : IDL.Nat64,
    'sequence' : IDL.Nat64,
  });
  return IDL.Service({
    'get_archived_count' : IDL.Func([IDL.Principal], [IDL.Nat64], ['query']),
    'get_archived_transactions' : IDL.Func(
        [IDL.Principal, IDL.Opt(IDL.Nat64), IDL.Nat64],
        [IDL.Vec(ArchivedTransaction)],
        ['query'],
      ),
    'get_parent_shard' : IDL.Func([], [IDL.Principal], ['query']),
    'get_stats' : IDL.Func([], [ArchiveStats], ['query']),
    'get_total_archived_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'receive_archive_batch' : IDL.Func(
        [IDL.Principal, IDL.Vec(TransactionToArchive)],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'set_next_archive' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'parent_shard_id' : IDL.Principal });
  return [InitArgs];
};
