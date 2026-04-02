PROHIBITED=["captcha_solving","mfa_bypass","exam_impersonation"]
def enforce_domain(url: str, allowed_domains: list[str]) -> bool: return any(domain in url for domain in allowed_domains)
