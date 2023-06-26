import { Button } from "primereact/button"
import { useContext } from "react";
import { ToastContext } from "../App";
import { backend } from "../rpc";

interface ComponentProps {
  value: string
}

export function CopyToClipboard(props: ComponentProps) {
  const toast = useContext(ToastContext);

  const copyToClipboard = () => {
    backend.copyToClipboard(props.value).then(() => {
      toast.showInfo("Copied to clipboard", "", 2000);
    }).catch((_err) => {
      toast.showError("Could not copy content to clipboard", "");
    });

  }

  return (
    <Button icon={"pi pi-copy"} onClick={copyToClipboard}
      tooltip={"Copy result to clipboard"}
      className={"p-button-rounded p-button-text"} aria-label={"Copy result to clipboard"}
      style={{ marginLeft: '10px' }}></Button>

  )
}
