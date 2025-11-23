# Rust参考文献

これはRustに関連する資料の読書リストです。過去にRustの設計に影響を与えた研究や、Rustに関する出版物が含まれています。

## 型システム

* [Alias burying](https://dl.acm.org/doi/10.1002/spe.370) - 似たようなことを試みて、断念しました。
* [External uniqueness is unique enough](https://lirias.kuleuven.be/retrieve/35835)
* [Macros that work together](https://www.cs.utah.edu/plt/publications/jfp12-draft-fcdf.pdf)
* [Making ad-hoc polymorphism less ad hoc](https://dl.acm.org/doi/10.1145/75277.75283)
* [Region based memory management in Cyclone](https://www.cs.umd.edu/projects/cyclone/papers/cyclone-regions.pdf)
* [Region Based Memory Management](https://www.cs.ucla.edu/~palsberg/tba/papers/tofte-talpin-iandc97.pdf)
* [Safe manual memory management in Cyclone](https://www.cs.umd.edu/projects/PL/cyclone/scp.pdf)
* [Skolem Normal Form](https://en.wikipedia.org/wiki/Skolem_normal_form)
* [Traits: composable units of behavior](http://scg.unibe.ch/archive/papers/Scha03aTraits.pdf)
* [Uniqueness and Reference Immutability for Safe Parallelism](https://research.microsoft.com/pubs/170528/msr-tr-2012-79.pdf)

## 並行性

* [A Java fork/join calamity](https://web.archive.org/web/20190904045322/http://www.coopsoft.com/ar/CalamityArticle.html) - Javaのfork/joinライブラリの批評、特に非正格計算へのワークスティーリングの適用について
* [Algorithms for scalable synchronization of shared-memory multiprocessors](https://www.cs.rochester.edu/u/scott/papers/1991_TOCS_synch.pdf)
* [Balanced work stealing for time-sharing multicores](https://web.njit.edu/~dingxn/papers/BWS.pdf)
* [Contention aware scheduling](https://www.blagodurov.net/files/a8-blagodurov.pdf)
* [Dynamic circular work stealing deque](https://patents.google.com/patent/US7346753B2/en) - Chase/Lev deque
* [Epoch-based reclamation](https://www.cl.cam.ac.uk/techreports/UCAM-CL-TR-579.pdf).
* [Language support for fast and reliable message passing in singularity OS](https://research.microsoft.com/pubs/67482/singsharp.pdf)
* [Non-blocking steal-half work queues](https://www.cs.bgu.ac.il/%7Ehendlerd/papers/p280-hendler.pdf)
* [Reagents: expressing and composing fine-grained concurrency](https://aturon.github.io/academic/reagents.pdf)
* [Scheduling multithreaded computations by work stealing](https://www.lri.fr/~cecile/ENSEIGNEMENT/IPAR/Exposes/cilk1.pdf)
* [Scheduling techniques for concurrent systems](https://www.stanford.edu/~ouster/cgi-bin/papers/coscheduling.pdf)
* [Singularity: rethinking the software stack](https://research.microsoft.com/pubs/69431/osr2007_rethinkingsoftwarestack.pdf)
* [The data locality of work stealing](http://www.aladdin.cs.cmu.edu/papers/pdfs/y2000/locality_spaa00.pdf)
* [Thread scheduling for multiprogramming multiprocessors](https://www.eecis.udel.edu/%7Ecavazos/cisc879-spring2008/papers/arora98thread.pdf)
* [Three layer cake for shared-memory programming](https://dl.acm.org/doi/10.1145/1953611.1953616)
* [Work-first and help-first scheduling policies for async-finish task parallelism](https://dl.acm.org/doi/10.1109/IPDPS.2009.5161079) - 完全正格ワークスティーリングよりも一般的

## その他

* [Composing High-Performance Memory Allocators](https://people.cs.umass.edu/~emery/pubs/berger-pldi2001.pdf)
* [Crash-only software](https://www.usenix.org/legacy/events/hotos03/tech/full_papers/candea/candea.pdf)
* [Reconsidering Custom Memory Allocation](https://people.cs.umass.edu/~emery/pubs/berger-oopsla2002.pdf)

## Rust*に関する*論文

* [GPU Programming in Rust: Implementing High Level Abstractions in a Systems
  Level
  Language](https://ieeexplore.ieee.org/document/6650903).
  Eric HolkによるGPU初期研究。
* [Parallel closures: a new twist on an old
  idea](https://www.usenix.org/conference/hotpar12/parallel-closures-new-twist-old-idea)
  - Rustについての論文ではありませんが、nmatsakisによるもの
* [Patina: A Formalization of the Rust Programming
  Language](https://dada.cs.washington.edu/research/tr/2015/03/UW-CSE-15-03-02.pdf).
  Eric Reedによる型システムの部分集合の初期形式化。
* [Experience Report: Developing the Servo Web Browser Engine using
  Rust](https://arxiv.org/abs/1505.07383). Lars Bergstromによる。
* [Implementing a Generic Radix Trie in
  Rust](https://michaelsproul.github.io/rust_radix_paper/rust-radix-sproul.pdf). Michael Sproulによる学部論文。
* [Reenix: Implementing a Unix-Like Operating System in
  Rust](https://scialex.github.io/reenix.pdf). Alex Lightによる学部論文。
* [Evaluation of performance and productivity metrics of potential programming languages in the HPC environment](https://github.com/1wilkens/thesis-ba).
  Florian Wilkensによる学士論文。C、Go、Rustを比較。
* [Nom, a byte oriented, streaming, zero copy, parser combinators library
  in Rust](http://spw15.langsec.org/papers/couprie-nom.pdf). Geoffroy CouprieによるVLCのための研究。
* [Graph-Based Higher-Order Intermediate
  Representation](https://compilers.cs.uni-saarland.de/papers/lkh15_cgo.pdf). Rust風の言語Impalaで実装された実験的IR。
* [Code Refinement of Stencil
  Codes](https://compilers.cs.uni-saarland.de/papers/ppl14_web.pdf). Impalaを使用した別の論文。
* [Parallelization in Rust with fork-join and
  friends](http://publications.lib.chalmers.se/records/fulltext/219016/219016.pdf). Linus Farnstrandの修士論文。
* [Session Types for
  Rust](https://munksgaard.me/papers/laumann-munksgaard-larsen.pdf). Philip Munksgaardの修士論文。Servoのための研究。
* [Ownership is Theft: Experiences Building an Embedded OS in Rust - Amit Levy, et. al.](https://amitlevy.com/papers/tock-plos2015.pdf)
* [You can't spell trust without Rust](https://faultlore.com/blah/papers/thesis.pdf). Aria Beingessnerの修士論文。
* [Rust-Bio: a fast and safe bioinformatics library](https://rust-bio.github.io/). Johannes Köster
* [Safe, Correct, and Fast Low-Level Networking](https://csperkins.org/research/thesis-msci-clipsham.pdf). Robert Clipshamの修士論文。
* [Formalizing Rust traits](https://open.library.ubc.ca/cIRcle/collections/ubctheses/24/items/1.0220521). Jonatan Milewskiの修士論文。
* [Rust as a Language for High Performance GC Implementation](https://dl.acm.org/doi/pdf/10.1145/3241624.2926707)
* [Simple Verification of Rust Programs via Functional Purification](https://github.com/Kha/electrolysis). Sebastian Ullrichの修士論文。
* [Writing parsers like it is 2017](http://spw17.langsec.org/papers/chifflier-parsing-in-2017.pdf) Pierre ChifflierとGeoffroy CouprieによるLangsec Workshopでの発表
* [The Case for Writing a Kernel in Rust](https://www.tockos.org/assets/papers/rust-kernel-apsys2017.pdf)
* [RustBelt: Securing the Foundations of the Rust Programming Language](https://plv.mpi-sws.org/rustbelt/popl18/)
* [Oxide: The Essence of Rust](https://arxiv.org/abs/1903.00982). Aaron Weiss、Olek Gierczak、Daniel Patterson、Nicholas D. Matsakis、Amal Ahmedによる。
