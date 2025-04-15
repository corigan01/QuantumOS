use criterion::{Criterion, criterion_group, criterion_main};
use mem::addr::PhysAddr;
use mem::page::PhysPage;
use mem::phys::{PhysMemoryEntry, PhysMemoryKind, PhysMemoryMap};
use mem::pmm::Pmm;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Physical Memory Manager");

    group.bench_function("Pmm allocate", |f| {
        const REAL_MEM_MAP: [PhysMemoryEntry; 7] = [
            PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: PhysAddr::new(0),
                end: PhysAddr::new(654336),
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: PhysAddr::new(654336),
                end: PhysAddr::new(654336 + 1024),
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: PhysAddr::new(983040),
                end: PhysAddr::new(983040 + 65536),
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Free,
                start: PhysAddr::new(1048576),
                end: PhysAddr::new(1048576 + 267255808),
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: PhysAddr::new(268304384),
                end: PhysAddr::new(268304384 + 131072),
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: PhysAddr::new(4294705152),
                end: PhysAddr::new(4294705152 + 262144),
            },
            PhysMemoryEntry {
                kind: PhysMemoryKind::Reserved,
                start: PhysAddr::new(1086626725888),
                end: PhysAddr::new(1086626725888 + 12884901888),
            },
        ];

        let mut mm = Box::new(PhysMemoryMap::<20>::new());

        for entry in REAL_MEM_MAP.iter() {
            mm.add_region(entry.clone()).unwrap();
        }

        let mut pmm = Pmm::new(&mm).unwrap();
        let mut pages_allocated = Box::new([PhysPage::new(0); 512]);

        f.iter(|| {
            for i in 0..512 {
                pages_allocated[i] = pmm.allocate_page().unwrap();
            }

            for i in 0..512 {
                pmm.free_page(pages_allocated[i]).unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
