// expand a period to ~TARGET blocks and write it in the corpus notation:
// bare lead-in term, numbered middle terms, bare lead-out term.
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
  // interior it is always its own piece, i.e. an elbow. Trim back to a run the
  // lead-out can sit inside.
  // (a walk with no run of 2 anywhere -- the all-elbow coil -- has no such tail;
  //  leave it alone rather than trimming it away to nothing)
  if(terms.some(t=>t[1]>=2))
    while(terms.length>1 && terms[terms.length-1][1]<2) terms.pop();
  // lead-in and lead-out must be single straight blocks
  if(terms[0][1]>1){ terms[0][1]--; terms.unshift([terms[0][0],1]); }
  const last=terms[terms.length-1];
  if(last[1]>1){ last[1]--; terms.push([last[0],1]); }
  return terms.map(([d,L],i)=> (i===0||i===terms.length-1) ? d : d+L).join(' ');
}
console.log(build(process.argv[2]));
