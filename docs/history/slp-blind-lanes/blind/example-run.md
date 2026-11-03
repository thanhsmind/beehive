# Blind lane run example-0001

<!--
The worked example of the dossier `bee blind check` reads, and the target of
that command's registry example. It is a WORKED SHAPE, not a record of a real
convergence: the ids are placeholders. It is checked by
`verbs::blind::tests::the_shipped_example_dossier_passes_this_door`, so the
shape taught here can never drift from the shape the door accepts.

Two rules the layout carries, both load-bearing:
  * the brief and every proposal ride in FENCED blocks, because a proposal is
    arbitrary prose and may quote a heading — outside a fence, a lane's own
    text would move the record's section boundaries;
  * every lane names its dispatch_id, so a later digest check reads the
    authoritative dispatch log instead of comparing one transcription against
    another.
-->

## Question

```
## Question
Which reader wins when two records claim the same path?

## Constraints
No new store. The answer stays inside the existing dispatch door.

## Read diet
- packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs

## Digest contract
Report the sha256 the dispatch record stamped.
```

## Lanes

### lane-a

- dispatch_id: 3f1c9a20-0000-4000-8000-000000000001
- brief_sha256: 9c1185a5c5e9fc54612808977ee8f548b2258d3100000000000000000000fa01
- role: advisor
- paths_read: packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs

```
The older lease wins on every read. A claim that arrives second never
outranks a live hold, so the reader needs no tie-break beyond the stamp
the record already carries.
```

### lane-b

- dispatch_id: 3f1c9a20-0000-4000-8000-000000000002
- brief_sha256: 9c1185a5c5e9fc54612808977ee8f548b2258d3100000000000000000000fa01
- role: advisor
- paths_read: packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs

```
Key the answer by path in a second index. The reader then answers in one
lookup instead of walking the lease set on every call.
```

## Cross-critiques

lane-a, handed lane-b verbatim, named the missing constraint: a second index
is a second place the truth lives, and the brief refused a new store.

lane-b, handed lane-a verbatim, named no missing context, so its objection
does not stand.

## Chosen

lane-a: the older lease wins on every read.

## Rejected

lane-b: it answers by adding a second store the brief ruled out.

## Citations

lane-a :: The older lease wins on every read
lane-b :: Key the answer by path in a second index

## Revisit trigger

lease-shape-changes__3f1c9a20
