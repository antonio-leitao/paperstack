let is_loading = $state(false);
let what = $state('Loading...');

export let LoadingState = {
    get is_loading() {
        return is_loading;
    },
    set is_loading(val) {
        is_loading = val;
    },
    get what() {
        return what;
    },
    set what(val) {
        what = val;
    },
    // Start loading with a specific title
    start(something) {
        is_loading = true;
        what = something;
    },
    // Stop loading and clear the title
    stop() {
        is_loading = false;
        what = '';
    },
    // Update title while ensuring loading state remains true
    lap(something) {
        is_loading = true;
        what = something;
    },
};
