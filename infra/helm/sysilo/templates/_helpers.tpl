{{/*
Expand the name of the chart.
*/}}
{{- define "sysilo.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "sysilo.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "sysilo.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "sysilo.labels" -}}
helm.sh/chart: {{ include "sysilo.chart" . }}
{{ include "sysilo.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "sysilo.selectorLabels" -}}
app.kubernetes.io/name: {{ include "sysilo.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Component-specific labels
*/}}
{{- define "sysilo.componentLabels" -}}
{{ include "sysilo.labels" . }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Component-specific selector labels
*/}}
{{- define "sysilo.componentSelectorLabels" -}}
{{ include "sysilo.selectorLabels" . }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Create the image path for a service
*/}}
{{- define "sysilo.image" -}}
{{- if .global.imageRegistry }}
{{- printf "%s/%s:%s" .global.imageRegistry .image .tag }}
{{- else }}
{{- printf "%s:%s" .image .tag }}
{{- end }}
{{- end }}

{{/*
Prometheus annotations for monitoring
*/}}
{{- define "sysilo.prometheusAnnotations" -}}
{{- if .Values.monitoring.prometheus.enabled }}
prometheus.io/scrape: "true"
prometheus.io/port: {{ .Values.monitoring.prometheus.port | quote }}
prometheus.io/path: {{ .Values.monitoring.prometheus.path | quote }}
{{- end }}
{{- end }}
