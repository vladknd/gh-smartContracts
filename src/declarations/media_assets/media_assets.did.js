export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'allowed_uploaders' : IDL.Vec(IDL.Principal) });
  const MediaType = IDL.Variant({
    'PDF' : IDL.Null,
    'Image' : IDL.Null,
    'Audio' : IDL.Null,
    'Other' : IDL.Null,
    'Video' : IDL.Null,
  });
  const FileMetadata = IDL.Record({
    'hash' : IDL.Text,
    'media_type' : MediaType,
    'size' : IDL.Nat64,
    'content_type' : IDL.Text,
    'filename' : IDL.Text,
    'chunk_count' : IDL.Nat32,
    'uploader' : IDL.Principal,
    'uploaded_at' : IDL.Nat64,
  });
  const UploadSession = IDL.Record({
    'uploaded_size' : IDL.Nat64,
    'session_id' : IDL.Text,
    'chunks_received' : IDL.Vec(IDL.Nat32),
    'media_type' : MediaType,
    'content_type' : IDL.Text,
    'expected_size' : IDL.Nat64,
    'filename' : IDL.Text,
    'uploader' : IDL.Principal,
    'started_at' : IDL.Nat64,
  });
  return IDL.Service({
    'add_allowed_uploader' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'cancel_upload' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'file_exists' : IDL.Func([IDL.Text], [IDL.Bool], ['query']),
    'finalize_upload' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'get_allowed_uploaders' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'get_file' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Vec(IDL.Nat8), 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_file_chunk' : IDL.Func(
        [IDL.Text, IDL.Nat32],
        [IDL.Opt(IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'get_file_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_file_metadata' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(FileMetadata)],
        ['query'],
      ),
    'get_file_url' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_total_storage' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_upload_session' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(UploadSession)],
        ['query'],
      ),
    'list_files' : IDL.Func(
        [IDL.Nat64, IDL.Nat64],
        [IDL.Vec(FileMetadata)],
        ['query'],
      ),
    'remove_allowed_uploader' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'start_upload' : IDL.Func(
        [IDL.Text, IDL.Text, MediaType, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'upload_chunk' : IDL.Func(
        [IDL.Text, IDL.Nat32, IDL.Vec(IDL.Nat8)],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'upload_file' : IDL.Func(
        [IDL.Text, IDL.Text, MediaType, IDL.Vec(IDL.Nat8)],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'allowed_uploaders' : IDL.Vec(IDL.Principal) });
  return [InitArgs];
};
