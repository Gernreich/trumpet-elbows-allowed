// expand a period to ~TARGET blocks and write it in the corpus notation:
// every term numbered. The bare lead-in and lead-out terms this used to write
// are not part of the notation any more -- a walk enters facing its first term
// and leaves facing its last -- so the two blocks they stood for are carried by
// the runs at each end instead.
const TARGET=196;
function steps(period){
  const out=[];
  for(const t of period.split(/\s+/)){
    const d=t[0], L=parseInt(t.slice(1)||'1',10);
    for(let i=0;i<L;i++) out.push(d);
  }
  return out;
}
function build(period){
  const u=steps(period), s=[];
  while(s.length<TARGET-1) s.push(u[s.length%u.length]);
  const terms=[]; let cur=s[0], n=1;
  for(let i=1;i<s.length;i++){ if(s[i]===cur) n++; else {terms.push([cur,n]); cur=s[i]; n=1;} }
  terms.push([cur,n]);
  // The bore's last block must not be a turn: with nothing after it to make it
  // interior it is always its own piece, i.e. stranded. Trim back to a run the
  // lead-out can sit inside.
  // (a walk with no run of 2 anywhere -- the all-stranded coil -- has no such tail;
  //  leave it alone rather than trimming it away to nothing)
  if(terms.some(t=>t[1]>=2))
    while(terms.length>1 && terms[terms.length-1][1]<2) terms.pop();
  // Lead-in and lead-out are still single straight blocks, but they cost no
  // term now. You start IN block 1 facing the first term, so the opening run
  // carries one move fewer than the period asks for and block 1 is the lead-in;
  // the closing run's last block is the lead-out, so it gives up a move too.
  if(terms[0][1]>1) terms[0][1]--;
  const last=terms[terms.length-1];
  if(last[1]>1) last[1]--;
  return terms.map(([d,L]) => d+L).join(' ');
}
// A missing argument reached build() and came back as a TypeError on
// `undefined.split` -- a stack trace where a usage line belongs. Every other
// tool here reads the corpus and needs no argument; these two take a period and
// said so nowhere.
if (!process.argv[2]) {
  console.error('usage: node tools/mknotation.js "<period>"   e.g. "N1 D4 E3 U4"');
  console.error('  Expands a period to about ' + TARGET + ' blocks in the corpus notation.');
  process.exit(2);
}
console.log(build(process.argv[2]));
