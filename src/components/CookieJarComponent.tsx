import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { useEffect, useState } from "react"
import { Collection, Cookie, CookieJar } from "../bindings"
import { catchError } from "../common/errorhandling";
import { backend } from "../rpc"
import { ActionDropdown } from "./ActionDropdown";
import { CopyToClipboard } from "./CopyToClipboard";
import DatePicker from "react-datepicker";
import "react-datepicker/dist/react-datepicker.css";
import { RelynxState, useRequestModelStore } from "../stores/requestStore";

interface ComponentProps {
}

export function CookieJarComponent(_props: ComponentProps) {

  const collection = useRequestModelStore((state: RelynxState) => state.currentCollection) as Collection;

  const [cookieJar, setCookieJar] = useState<CookieJar>({
    path: null,
    cookies: []
  });

  useEffect(() => {
    if (!collection) {
      return
    }
    backend.get_cookie_jar(collection).then((cookieJar: CookieJar) => {
      setCookieJar(cookieJar);
    }).catch(catchError);
  }, [collection?.path]);

  const updateCookieDomain = (index: number, newDomain: string) => {
    updateCookieJarCookie(index, { domain: newDomain });
  }

  const updateCookiePath = (index: number, newPath: string) => {
    updateCookieJarCookie(index, { path: newPath });
  }

  const updateCookieName = (index: number, newName: string) => {
    updateCookieJarCookie(index, { name: newName });
  }

  const updateCookieValue = (index: number, newValue: string) => {
    updateCookieJarCookie(index, { value: newValue });
  }

  const addCookie = () => {
    let newCookieJar = structuredClone(cookieJar);
    newCookieJar.cookies.push({
      domain: "",
      path: "",
      name: "",
      value: "",
      expires: new Date(2100, 0, 1).toUTCString()
    });
    updateCookieJar(newCookieJar);
  }

  const removeCookie = (index: number) => {
    let newCookieJar = structuredClone(cookieJar);
    newCookieJar.cookies.splice(index, 1);
    updateCookieJar(newCookieJar);
  }

  const updateCookieJarCookie = (index: number, partial: Partial<Cookie>) => {
    let newCookie = { ...cookieJar.cookies[index], ...partial };
    let newCookieJar = structuredClone(cookieJar);
    newCookieJar.cookies[index] = newCookie;
    updateCookieJar(newCookieJar);
  }

  const setExpiresDate = (index: number, date: Date) => {
    updateCookieJarCookie(index, { expires: date.toUTCString() })
  }

  const updateCookieJar = (newCookieJar: CookieJar) => {
    backend.save_cookie_jar(collection, newCookieJar).then(() => {
      setCookieJar(newCookieJar);
    }).catch(catchError);
  }

  function isValidDate(d: any) {
    // @ts-ignore
    return d instanceof Date && !isNaN(d);
  }

  const expiresDate = (cookie: Cookie): Date => {
    let date = new Date(cookie.expires);
    if (isValidDate(date)) {
      return date;
    }

    date = new Date(Date.parse(cookie.expires));

    if (isValidDate(date)) {
      return date;
    }

    return new Date();
  }

  return (
    <div className={'fade-in-fast'}
      style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '100%' }}>

      <h1 style={{ marginTop: '20px', marginBottom: '50px' }}>Cookie Jar</h1>

      {
        cookieJar.path !== null &&
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
          <label>Cookie Jar Path: </label>
          <div style={{ display: 'flex', alignItems: 'center', marginTop: '5px' }}>
            <InputText tooltip={cookieJar.path} value={cookieJar.path} style={{ width: '100%', direction: 'rtl' }} />
            <CopyToClipboard value={cookieJar.path} tooltip={"Copy path to cookie jar"} />
          </div>
        </div>
      }
      <p style={{ marginTop: '20px', marginBottom: '20px', textAlign: 'left' }}>
        The cookie jar is a file that contains cookies that will be sent with a request if the domain and path matches.
        All cookies returned by a response that should be set will also be saved in this cookie jar folder.
        <br />
        <br />
        At most 300 cookies can be saved within this file.
        <br />
        <br />
        If you do not want to send cookies with a request you can set the request settings to not allow the cookie jar.
        <br />
        <br />
        In case you want to specify cookies manually for a request you can just set the `Cookie` header and pass key value pairs like this:
        <br />
        <br />
        `Cookie: keyone=valueone; keytwo=valuetwo`.
      </p>
      <div style={{ marginTop: '20px', width: '100%' }}>
        {
          cookieJar.cookies.length === 0 &&
          <p style={{ textAlign: 'left' }}>No cookies present yet. Add a new cookie that will be sent within the request if the request's domain matches the cookie's domain.</p>
        }
        {
          cookieJar.cookies.length > 0 && <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '10px', marginBottom: '10px' }}>
            <div className="cookie-table-header">Domain</div>
            <div className="cookie-table-header">Path</div>
            <div className="cookie-table-header">Name</div>
            <div className="cookie-table-header">Value</div>
            <div className="cookie-table-header">Expires</div>
            <div className="cookie-table-header-actions"></div>
          </div>

        }

        {
          cookieJar.cookies.map((cookie: Cookie, index: number) => {
            return <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: '10px' }}>
              <InputText className="cookie-entry" value={cookie.domain} onChange={(event) => updateCookieDomain(index, event.target.value)} />
              <InputText className="cookie-entry" value={cookie.path} onChange={(event) => updateCookiePath(index, event.target.value)} />
              <InputText className="cookie-entry" value={cookie.name} onChange={(event) => updateCookieName(index, event.target.value)} />
              <InputText className="cookie-entry" value={cookie.value} onChange={(event) => updateCookieValue(index, event.target.value)} />
              <DatePicker className="cookie-entry" dateFormat={"E, dd MMM yyyy HH:mm:ss z"} selected={expiresDate(cookie)} showTimeSelect onChange={(date: Date) => setExpiresDate(index, date)} />
              <ActionDropdown className="cookie-entry-actions" styles={{}}>
                <Button icon={'pi pi-trash'} className={'p-button p-button-text'}
                  label={"Remove"}
                  onClick={() => removeCookie(index)} />
              </ActionDropdown>
            </div>
          })
        }
      </div>
      <Button icon={'pi pi-plus'} label={"Add Cookie"}
        className={'p-button-raised p-button-text'}
        style={{ marginTop: '30px', maxWidth: '180px' }} onClick={addCookie} />
    </div>

  )
}
