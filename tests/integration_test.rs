use rand::{
    distributions::Standard,
    Rng,
};
use smallvec::smallvec;

use n5::prelude::*;

fn test_read_write<T, N5: N5Reader + N5Writer>(n: &N5, compression: &CompressionType, dim: usize)
where
    T: 'static + std::fmt::Debug + ReflectedType + PartialEq + Default,
    rand::distributions::Standard: rand::distributions::Distribution<T>,
    VecDataChunk<T>: n5::ReadableDataChunk + n5::WriteableDataChunk,
{
    let chunk_size: ChunkCoord = (1..=dim as u32).rev().map(|d| d * 5).collect();
    let data_attrs = DatasetAttributes::new(
        (1..=dim as u64).map(|d| d * 100).collect(),
        chunk_size.clone(),
        T::VARIANT,
        compression.clone(),
    );
    let numel = data_attrs.get_chunk_num_elements();
    let rng = rand::thread_rng();
    let chunk_data: Vec<T> = rng.sample_iter(&Standard).take(numel).collect();

    let chunk_in = SliceDataChunk::new(chunk_size, smallvec![0; dim], chunk_data);

    let path_name = "test/dataset/group";

    n.create_dataset(path_name, &data_attrs)
        .expect("Failed to create dataset");
    n.write_chunk(path_name, &data_attrs, &chunk_in)
        .expect("Failed to write chunk");

    let chunk_data = chunk_in.into_data();

    let chunk_out = n
        .read_chunk::<T>(path_name, &data_attrs, smallvec![0; dim])
        .expect("Failed to read chunk")
        .expect("Chunk is empty");
    assert_eq!(chunk_out.get_data(), &chunk_data[..]);

    let mut into_chunk = VecDataChunk::new(smallvec![0; dim], smallvec![0; dim], vec![]);
    n.read_chunk_into(path_name, &data_attrs, smallvec![0; dim], &mut into_chunk)
        .expect("Failed to read chunk")
        .expect("Chunk is empty");
    assert_eq!(into_chunk.get_data(), &chunk_data[..]);

    n.remove(path_name).unwrap();
}

fn test_all_types<N5: N5Reader + N5Writer>(n: &N5, compression: &CompressionType, dim: usize) {
    test_read_write::<u8, _>(n, compression, dim);
    test_read_write::<u16, _>(n, compression, dim);
    test_read_write::<u32, _>(n, compression, dim);
    test_read_write::<u64, _>(n, compression, dim);
    test_read_write::<i8, _>(n, compression, dim);
    test_read_write::<i16, _>(n, compression, dim);
    test_read_write::<i32, _>(n, compression, dim);
    test_read_write::<i64, _>(n, compression, dim);
    test_read_write::<f32, _>(n, compression, dim);
    test_read_write::<f64, _>(n, compression, dim);
}

fn test_n5_filesystem_dim(dim: usize) {
    let dir = tempdir::TempDir::new("rust_n5_integration_tests").unwrap();

    let n = N5Filesystem::open_or_create(dir.path()).expect("Failed to create N5 filesystem");
    test_all_types(
        &n,
        &CompressionType::Raw(compression::raw::RawCompression::default()),
        dim,
    );
}

#[test]
fn test_n5_filesystem_dims() {
    for dim in 1..=5 {
        test_n5_filesystem_dim(dim);
    }
}

fn test_all_compressions<N5: N5Reader + N5Writer>(n: &N5) {
    test_all_types(
        n,
        &CompressionType::Raw(compression::raw::RawCompression::default()),
        3,
    );
    #[cfg(feature = "bzip")]
    test_all_types(
        n,
        &CompressionType::Bzip2(compression::bzip::Bzip2Compression::default()),
        3,
    );
    #[cfg(feature = "gzip")]
    test_all_types(
        n,
        &CompressionType::Gzip(compression::gzip::GzipCompression::default()),
        3,
    );
    #[cfg(feature = "lz")]
    test_all_types(
        n,
        &CompressionType::Lz4(compression::lz::Lz4Compression::default()),
        3,
    );
    #[cfg(feature = "xz")]
    test_all_types(
        n,
        &CompressionType::Xz(compression::xz::XzCompression::default()),
        3,
    );
}

#[test]
fn test_n5_filesystem_compressions() {
    let dir = tempdir::TempDir::new("rust_n5_integration_tests").unwrap();

    let n = N5Filesystem::open_or_create(dir.path()).expect("Failed to create N5 filesystem");
    test_all_compressions(&n)
}
