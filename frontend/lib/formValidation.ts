import { useCallback, useEffect, useRef, useState } from 'react';

export type Errors<T> = Partial<Record<keyof T, string | undefined>>;

type ValidateFn<T> = (values: T) => Errors<T> | Promise<Errors<T>>;

export function useFormValidation<T extends Record<string, any>>(opts: {
  initialValues: T;
  validate?: ValidateFn<T>;
  onSubmit: (values: T) => void | Promise<void>;
  debounceMs?: number;
}) {
  const { initialValues, validate, onSubmit, debounceMs = 300 } = opts;

  const [values, setValues] = useState<T>(initialValues);
  const [errors, setErrors] = useState<Errors<T>>({});
  const [touched, setTouched] = useState<Record<string, boolean>>({});
  const validatingRef = useRef<number | null>(null);
  const latestValidate = useRef(validate);
  latestValidate.current = validate;

  const runValidation = useCallback(async (vals: T) => {
    if (!latestValidate.current) return {};
    const res = latestValidate.current(vals);
    const resolved = res instanceof Promise ? await res : res;
    return resolved || {};
  }, []);

  useEffect(() => {
    // real-time validation with debounce
    if (!validate) return;
    if (validatingRef.current) clearTimeout(validatingRef.current);
    validatingRef.current = window.setTimeout(async () => {
      const nextErrors = await runValidation(values);
      setErrors(nextErrors);
      validatingRef.current = null;
    }, debounceMs);
    return () => validatingRef.current && clearTimeout(validatingRef.current);
  }, [values, validate, debounceMs, runValidation]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
      const { name, type } = e.target as HTMLInputElement;
      let value: any = (e.target as HTMLInputElement).value;
      if (type === 'checkbox') value = (e.target as HTMLInputElement).checked;
      setValues((v) => ({ ...v, [name]: value }));
    },
    [],
  );

  const handleBlur = useCallback((e: React.FocusEvent<any>) => {
    const name = e.target.name;
    setTouched((t) => ({ ...t, [name]: true }));
  }, []);

  const setFieldValue = useCallback((name: keyof T, value: any) => {
    setValues((v) => ({ ...v, [name]: value }));
  }, []);

  const handleSubmit = useCallback(
    async (e?: React.FormEvent) => {
      if (e && typeof e.preventDefault === 'function') e.preventDefault();
      if (validate) {
        const next = await runValidation(values);
        setErrors(next);
        const hasErrors = Object.values(next).some(Boolean);
        if (hasErrors) return { success: false, errors: next };
      }
      await onSubmit(values);
      return { success: true };
    },
    [onSubmit, runValidation, validate, values],
  );

  return {
    values,
    setValues,
    errors,
    setErrors,
    touched,
    handleChange,
    handleBlur,
    handleSubmit,
    setFieldValue,
  } as const;
}

// Basic validators
export const validators = {
  required: (v: any) => (v === undefined || v === null || v === '' ? 'Required' : undefined),
  url: (v: any) => {
    if (!v) return undefined;
    try {
      // eslint-disable-next-line no-new
      new URL(v);
      return undefined;
    } catch (e) {
      return 'Invalid URL';
    }
  },
  semver: (v: any) => {
    if (!v) return undefined;
    const semverRe = /^\d+\.\d+\.\d+(-[0-9A-Za-z-.]+)?(\+[0-9A-Za-z-.]+)?$/;
    return semverRe.test(String(v)) ? undefined : 'Invalid semver (e.g. 1.2.3)';
  },
  // Stellar public key (starts with G, 56 chars)
  stellarPublicKey: (v: any) => {
    if (!v) return undefined;
    const s = String(v);
    return s.length === 56 && s[0] === 'G' ? undefined : 'Invalid Stellar public key';
  },
};

export default useFormValidation;
