let x = $state(0);
let y = $state(0);
let paper = $state(null);
let handleDelete = $state(null);
let show = $state(false);
let stack = $state(null);
let stackRename = $state(null);
let stackMerge = $state(null);
let stackDupe = $state(null);

export let ContextState = {
  get stackRename() {
    return stackRename;
  },
  set stackRename(val) {
    stackRename = val;
  },
  get stackMerge() {
    return stackMerge;
  },
  set stackMerge(val) {
    stackMerge = val;
  },
  get stackDupe() {
    return stackDupe;
  },
  set stackDupe(val) {
    stackDupe = val;
  },
  get show() {
    return show;
  },
  set show(val) {
    show = val;
  },
  get paper() {
    return paper;
  },
  set paper(val) {
    paper = val;
  },
  get stack() {
    return stack;
  },
  set stack(val) {
    stack = val;
  },
  get x() {
    return x;
  },
  set x(val) {
    x = val;
  },
  get y() {
    return y;
  },
  set y(val) {
    y = val;
  },
  get handleDelete() {
    return handleDelete;
  },
  set handleDelete(val) {
    handleDelete = val;
  },
  open_paper(xx, yy, ppaper, hhandleDelete) {
    show = true;
    x = xx;
    y = yy;
    paper = ppaper;
    stack = null;
    handleDelete = hhandleDelete;
  },
  open_stack(xx, yy, sstack) {
    show = true;
    x = xx;
    y = yy;
    paper = null;
    stack = sstack;
  },
  close() {
    show = false;
  },
};
