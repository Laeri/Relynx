import { useEffect, useState } from "react";
import { RequestModel, Header } from '../../bindings';
import BodySelectMenu, { BodyType, BodyTypes, TextBodyTypes, toMimeType } from "./BodySelectMenu";
import { DataSourceFromFilepath, get_content_type, hasContentType, newMultipartPart, RequestBodyMultipart, RequestBodyRaw, RequestBodyUrlEncoded, setHeader } from "../../model/request";
import { MultipartBody } from "./MultipartBody";
import { UrlEncodedBody } from "./UrlEncodedBody";
import { RawTypes, TextBody } from "./TextBody";
import { Divider } from "primereact/divider";

interface ComponentProps {
  updateRequest: (newRequest: RequestModel) => void,
  request: RequestModel
}

export type RawType = "text" | "file";

export function RequestBodyComp(props: ComponentProps) {

  //const [formProperties, setFormProperties] = useState<FormProperty[]>([new FormProperty()]);
  const [currentBodyType, setCurrentBodyType] = useState<BodyType>(BodyTypes.no_body);
  const [_isText, setIsText] = useState<boolean>(false);


  const [multipartBody, setMultipartBody] = useState<RequestBodyMultipart>({ Multipart: { boundary: "--boundary--", parts: [] } });
  const [urlEncodedBody, setUrlEncodedBody] = useState<RequestBodyUrlEncoded>({ UrlEncoded: { url_encoded_params: [] } });

  const [rawTextBody, setRawTextBody] = useState<RequestBodyRaw>({ Raw: { data: { Raw: "" } } });
  const [rawTextFileBody, setRawTextFileBody] = useState<RequestBodyRaw>({ Raw: { data: { FromFilepath: "" } } });
  const [rawType, setRawType] = useState<RawType>("text");

  const updateBodyType = () => {

    if (props.request.body == "None") {
      setCurrentBodyType(BodyTypes.no_body);
      return
    }
    // @TODO: all from file?
    if ((props.request.body as RequestBodyMultipart).Multipart !== undefined) {
      setCurrentBodyType(BodyTypes.multipart_form);
      setMultipartBody((props.request.body as RequestBodyMultipart));
      return
    }
    // @TODO: from file?
    if ((props.request.body as RequestBodyUrlEncoded).UrlEncoded !== undefined) {
      setCurrentBodyType(BodyTypes.form_urlencoded);
      setUrlEncodedBody((props.request.body as RequestBodyUrlEncoded));
      return
    }
    // these are only set if some body is present, otherwise the request will have these headers but body None
    if ((props.request.body as RequestBodyRaw).Raw !== undefined) {
      let rawBody = (props.request.body as RequestBodyRaw);
      let isFile = (rawBody.Raw.data as DataSourceFromFilepath).FromFilepath !== undefined;
      if (isFile) {
        setRawType("file");
        setRawTextFileBody(rawBody);
      } else {
        setRawType("text");
        setRawTextBody(rawBody);
      }
      if (hasContentType(props.request, "application/json")) {
        setCurrentBodyType(BodyTypes.json);
      } else if (hasContentType(props.request, "text/plain")) {
        setCurrentBodyType(BodyTypes.plain_text);
      } else if (hasContentType(props.request, "text/yaml")) {
        setCurrentBodyType(BodyTypes.yaml);
      } else if (hasContentType(props.request, "application/xml")) {
        setCurrentBodyType(BodyTypes.xml);
      } else if (hasContentType(props.request, "application/octet-stream") !== undefined) {
        setCurrentBodyType(BodyTypes.binary_file);
      } else if (get_content_type(props.request) !== undefined) {
        setCurrentBodyType(BodyTypes.other);
      }
    }
  }

  useEffect(() => {
    updateBodyType();
  }, [props.request]);

  const updateContentType = (newType: BodyType, newRequest: RequestModel) => {
    let newMimeType = toMimeType(newType);

    if (newMimeType) {
      let header: Header = { key: "Content-Type", value: newMimeType, active: true };
      setHeader(newRequest, header);
    }
  }

  const updateType = (newType: BodyType, isText: boolean) => {

    let newRequest = structuredClone(props.request);

    updateContentType(newType, newRequest);

    setCurrentBodyType(newType);
    setIsText(isText);

    if (newType === BodyTypes.multipart_form) {
      updateBodyMultipart(multipartBody, newRequest);
      return
    }

    if (TextBodyTypes.includes(newType)) {
      if (rawType === "text") {
        updateRawBody(rawTextBody, newRequest);
      } else {
        updateRawBody(rawTextFileBody, newRequest);
      }
      return
    }
    if (newType === BodyTypes.form_urlencoded) {
      // @TODO
    }

    if (newType === BodyTypes.binary_file) {
      // @TODO
    }

    if (newType === BodyTypes.no_body) {
      newRequest.body = "None";
      props.updateRequest(newRequest);
    }

  }

  const addMultipartPart = () => {
    let newBody = structuredClone(multipartBody);
    newBody.Multipart.parts.push(newMultipartPart());
    setMultipartBody(newBody);
    let newRequest = structuredClone(props.request);
    newRequest.body = newBody;
    props.updateRequest(newRequest)
  }

  const updateBodyMultipart = (newBody: RequestBodyMultipart, newRequest?: RequestModel) => {
    if (!newRequest) {
      newRequest = structuredClone(props.request);
    }
    setMultipartBody(newBody);
    newRequest.body = newBody;
    let index = newRequest.headers.findIndex((header: Header) => header.key.toLowerCase() === "content-type");
    // @TODO: sanitize boundary
    let header = { key: 'Content-Type', value: `multipart/form-data; boundary="${newBody.Multipart.boundary}"`, active: true };
    if (index !== -1) {
      newRequest.headers[index] = header;
    } else {
      newRequest.headers.push(header);
    }
    props.updateRequest(newRequest);
  }

  const updateRawBody = (newBody: RequestBodyRaw, newRequest?: RequestModel) => {
    if (!newRequest) {
      newRequest = structuredClone(props.request);
    }
    newRequest.body = newBody;
    props.updateRequest(newRequest);
  }

  const updateRawType = (newRawType: RawType, newRequest?: RequestModel) => {
    if (!newRequest) {
      newRequest = structuredClone(props.request)
    }

    setRawType(newRawType);
    if (newRawType === RawTypes.text) {
      updateRawBody(rawTextBody, newRequest);
    } else {
      updateRawBody(rawTextFileBody, newRequest);
    }
  }

  return (
    <>
      {
        < div className="headers-block" style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }
        }>
          <h2>Body</h2>
          <BodySelectMenu style={{ marginTop: '40px', marginBottom: '20px' }} currentType={currentBodyType} setNewType={(newType: BodyType, isText: boolean) => updateType(newType, isText)} />
          <Divider type="solid" />
          {
            (TextBodyTypes.includes(currentBodyType)) &&
            <TextBody updateRawType={updateRawType} bodyFile={rawTextFileBody} bodyText={rawTextBody} rawType={rawType} updateBody={updateRawBody} />
          }
          {
            (currentBodyType === BodyTypes.form_urlencoded) &&
            <UrlEncodedBody body={urlEncodedBody} />
          }

          {
            (currentBodyType === BodyTypes.multipart_form) &&
            <MultipartBody body={multipartBody} addPart={addMultipartPart} updateBodyMultipart={updateBodyMultipart} />
          }

          {
            (currentBodyType === BodyTypes.form_urlencoded) &&
            <UrlEncodedBody body={urlEncodedBody} />
          }

          {
            (currentBodyType === BodyTypes.no_body) &&
            <p style={{ marginTop: '30px' }}>No request body present</p>
          }
        </div >
      }
    </>
  )
}
