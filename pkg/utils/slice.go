package utils

func ReverseStringSlice(ss []string) []string {
	for i := len(ss)/2 - 1; i >= 0; i-- {
		opp := len(ss) - 1 - i
		ss[i], ss[opp] = ss[opp], ss[i]
	}
	return ss
}

func IsInStringSlice(ss []string, el string) bool {
	for _, s := range ss {
		if s == el {
			return true
		}
	}
	return false
}
