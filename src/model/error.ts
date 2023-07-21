
export interface FrontendError {
  kind: string,
  message: string
};

export type BackendError = string;

export function newFrontendError(): FrontendError {
  return {
    kind: "Generic",
    message: ""
  };
}

export function NewFError(id: string, title: string, userMsg: string, errorMsg: string): FrontendError {
  let frontendError = newFrontendError();
  return frontendError;
}

export type CancellationToken = {
  cancelled: boolean
}
