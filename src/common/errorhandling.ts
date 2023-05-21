import { ToastContext } from "../App";
import { FrontendError } from "../bindings";

export function catchError(toast: ToastContext): (err: FrontendError) => void {
  return (err: FrontendError) => {
    console.error('catch error: ', err)
    toast.showError('', err);
  }
}

export type FError = FrontendError | undefined

export function displayAndLogErr(error: FrontendError, toast: ToastContext) {
  //@TODO: use log plugin functions
  /* LogFrontendError(error).then(() => {
    // ignored
  }).catch((err: string) => {
    console.error("Could not log frontend error correctly in displayAndLogErr!: ", err);
  }) */
  console.error(`Error id: ${error.Id}, title: ${error.Title}, userMsg: ${error.UserMsg}, errorMsg: ${error.Error}`);
  toast.showError(error.Title, error.UserMsg);
}

/*@TODO export function formatParseErrorsMsg(parseErrors: HttpRequestParseError[]): string {
  let result = "Folowing errors occurred: \n\n";

  result += parseErrors.map((parseError: HttpRequestParseError) => {
    return `${parseError.Item}:${parseError.Path} Request file is malformed -> ${parseError.Description} \n`
  });
  return result;
} */
