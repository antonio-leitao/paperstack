const DialogActions = {
  CANCEL: 'cancel',
  REPLACE: 'replace',
  KEEP: 'keep',
  UPDATE: 'update'
};

// Base dialog state with proper structure for each dialog type
let dialogState = $state({
  isOpen: false,
  type: null,
  // Loading dialog fields
  message: '',
  // Confirmation dialog fields
  title: '',
  field: '',
  value: '',
  // Duplicate dialog fields
  duplicatePaper: null,
  newPaper: null,
  // Common fields
  resolve: null
});

export const DialogStore = {
  get state() {
    return dialogState;
  },

  start(message) {
    dialogState = {
      ...dialogState,
      type: "loading",
      message: message,
      isOpen: true
    };
  },

  async confirm({ title, field, value = '' }) {
    return new Promise((resolve) => {
      dialogState = {
        ...dialogState,
        type: "confirmation",
        title: title,
        field: field,
        value: value,
        isOpen: true,
        resolve
      };
    });
  },

  async handleDuplicate(paper, duplicatePaper) {
    return new Promise((resolve) => {
      dialogState = {
        ...dialogState,
        type: "duplicate",
        newPaper: paper,
        duplicatePaper,
        isOpen: true,
        resolve
      };
    });
  },

  close() {
    if (dialogState.resolve) {
      dialogState.resolve(DialogActions.CANCEL);
    }
    dialogState = {
      ...dialogState,
      isOpen: false
    };
  },

  confirm_action() {
    if (dialogState.resolve) {
      dialogState.resolve(true);
    }
    dialogState = {
      ...dialogState,
      isOpen: false
    };
  },

  selectAction(action) {
    if (dialogState.resolve) {
      dialogState.resolve(action);
    }
    dialogState = {
      ...dialogState,
      isOpen: false
    };
  }
};

export { DialogActions };
