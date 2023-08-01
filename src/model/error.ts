
export interface FrontendError {
  id: string,
  title: string,
  message: string,
  devMsg: string
};

export function NewFError(id: string, title: string, message: string, devMsg: string): FrontendError {
  return {
    id,
    title,
    message,
    devMsg
  };
}

export type CancellationToken = {
  cancelled: boolean
}
