export const onCtrlEnter = (event: KeyboardEvent, callback: () => void) => {
    if (event.code === 'Enter' && event.ctrlKey) {
        callback();
    }
}
